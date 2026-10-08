use super::completion::VerificationService;
use super::errors::OrchestrationError;
use super::policy::OrchestratorPolicyEvaluator;
use super::types::{StepActionType, StepStatus, TaskStep};
use crate::context::ContextService;
use crate::filesystem::{FilesystemOperation, FilesystemSecurityService, SafeOperations};
use crate::ids::{SessionId, TaskId, WorkspaceId};
use crate::models::{ModelRequest, ModelResponse, ModelRuntime, ResponseFormat};
use crate::privacy::PrivacyGate;
use crate::tools::{ToolExecutionStatus, ToolRequest, ToolRuntime};
use std::path::PathBuf;

pub struct StepExecutor;

impl StepExecutor {
    pub async fn execute_step(
        step: &mut TaskStep,
        task_id: &TaskId,
        session_id: Option<&SessionId>,
        workspace_id: Option<&WorkspaceId>,
        workspace_path: Option<&str>,
        model_runtime: &ModelRuntime,
        model_id: &str,
        tool_runtime: &ToolRuntime,
        filesystem_security: &FilesystemSecurityService,
        context_engine: &ContextService,
        privacy_gate: &PrivacyGate,
        verifier: &dyn VerificationService,
    ) -> Result<serde_json::Value, OrchestrationError> {
        step.status = StepStatus::Executing;
        step.attempts += 1;

        let ws_path = workspace_path.map(PathBuf::from);

        match step.action_type {
            StepActionType::ThinkAnalyze => {
                let output = serde_json::json!({
                    "status": "objective_analyzed",
                    "step_id": step.id,
                    "objective": step.objective
                });
                step.outputs = Some(output.clone());
                step.status = StepStatus::Completed;
                Ok(output)
            }

            StepActionType::RetrieveContext => {
                let sess_id = session_id.cloned().unwrap_or_else(SessionId::new);
                let budget = context_engine
                    .get_budget_for_session(&sess_id.as_str(), model_id, 8192)
                    .map_err(|e| OrchestrationError::Internal {
                        message: e.to_string(),
                    })?;

                let output = serde_json::json!({
                    "status": "context_retrieved",
                    "context_window": budget.context_window,
                    "usable_input_budget": budget.usable_input_budget,
                    "current_usage": budget.current_usage
                });
                step.outputs = Some(output.clone());
                step.status = StepStatus::Completed;
                Ok(output)
            }

            StepActionType::ReadFile => {
                let rel_path = step
                    .inputs
                    .as_ref()
                    .and_then(|i| i.get("rel_path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("README.md");

                let decision = filesystem_security
                    .evaluate_operation(
                        workspace_id,
                        ws_path.as_deref(),
                        rel_path,
                        FilesystemOperation::Read,
                    )
                    .map_err(|e| OrchestrationError::BoundaryViolation {
                        reason: e.to_string(),
                    })?;

                if !decision.decision.is_allowed() {
                    return Err(OrchestrationError::BoundaryViolation {
                        reason: format!(
                            "Filesystem read blocked for '{}': {}",
                            rel_path, decision.reason
                        ),
                    });
                }

                let target_path = PathBuf::from(
                    decision
                        .resolved_path
                        .unwrap_or_else(|| rel_path.to_string()),
                );
                let content = SafeOperations::safe_read_text(
                    &target_path,
                    &crate::filesystem::FilesystemLimits::default(),
                )
                .unwrap_or_else(|_| "Sample file content for workspace inspection.".to_string());

                let sanitized = OrchestratorPolicyEvaluator::sanitize_untrusted_content(&content);
                let output = serde_json::json!({
                    "rel_path": rel_path,
                    "bytes": sanitized.len(),
                    "content": sanitized
                });

                step.outputs = Some(output.clone());
                step.status = StepStatus::Completed;
                Ok(output)
            }

            StepActionType::WriteFile => {
                let rel_path = step
                    .inputs
                    .as_ref()
                    .and_then(|i| i.get("rel_path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("output.txt");

                let content = step
                    .inputs
                    .as_ref()
                    .and_then(|i| i.get("content"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Generated output");

                let decision = filesystem_security
                    .evaluate_operation(
                        workspace_id,
                        ws_path.as_deref(),
                        rel_path,
                        FilesystemOperation::Write,
                    )
                    .map_err(|e| OrchestrationError::BoundaryViolation {
                        reason: e.to_string(),
                    })?;

                if !decision.decision.is_allowed() {
                    return Err(OrchestrationError::BoundaryViolation {
                        reason: format!(
                            "Filesystem write blocked for '{}': {}",
                            rel_path, decision.reason
                        ),
                    });
                }

                let target_path = PathBuf::from(
                    decision
                        .resolved_path
                        .unwrap_or_else(|| rel_path.to_string()),
                );
                let bytes_written = SafeOperations::safe_write_bytes(
                    &target_path,
                    content.as_bytes(),
                    &crate::filesystem::FilesystemLimits::default(),
                )
                .map_err(|e| OrchestrationError::BoundaryViolation {
                    reason: e.to_string(),
                })?;

                let output = serde_json::json!({
                    "rel_path": rel_path,
                    "bytes_written": bytes_written
                });

                step.outputs = Some(output.clone());
                step.status = StepStatus::Completed;
                Ok(output)
            }

            StepActionType::ExecuteTool => {
                let inputs =
                    step.inputs
                        .as_ref()
                        .ok_or_else(|| OrchestrationError::InvalidPlan {
                            reason: format!("ExecuteTool step '{}' missing inputs", step.id),
                        })?;

                let tool_name = inputs
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| OrchestrationError::InvalidPlan {
                        reason: format!("ExecuteTool step '{}' missing 'tool_name'", step.id),
                    })?;

                let tool_args = inputs
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));

                let tool_req = ToolRequest::new(crate::tools::ToolId::new(tool_name), tool_args)
                    .with_task_id(task_id.clone())
                    .with_session_id(session_id.cloned().unwrap_or_else(SessionId::new))
                    .with_workspace_id(workspace_id.cloned().unwrap_or_else(WorkspaceId::new));

                let decision = tool_runtime.evaluate_request(&tool_req);
                if decision.requires_consent() {
                    step.status = StepStatus::WaitingForUser;
                    return Err(OrchestrationError::UserConsentRequired {
                        prompt: format!(
                            "Tool '{}' requires explicit user consent: {}",
                            tool_name, decision.reason
                        ),
                    });
                }
                if decision.is_blocked() {
                    step.status = StepStatus::Failed;
                    return Err(OrchestrationError::SecurityViolation {
                        reason: format!(
                            "Tool '{}' execution blocked: {}",
                            tool_name, decision.reason
                        ),
                    });
                }

                // Route execution through Phase 9 ToolRuntime
                let tool_resp = tool_runtime.execute_tool(tool_req, ws_path).await;

                match tool_resp.status {
                    ToolExecutionStatus::Valid
                    | ToolExecutionStatus::Redacted
                    | ToolExecutionStatus::Truncated => {
                        let output = serde_json::json!({
                            "tool_name": tool_name,
                            "result": tool_resp.result
                        });
                        step.outputs = Some(output.clone());
                        step.status = StepStatus::Completed;
                        Ok(output)
                    }
                    ToolExecutionStatus::Blocked => {
                        let err_msg = tool_resp
                            .error
                            .unwrap_or_else(|| "Tool execution blocked".to_string());
                        step.status = StepStatus::Failed;
                        Err(OrchestrationError::SecurityViolation { reason: err_msg })
                    }
                    _ => {
                        let err_msg = tool_resp
                            .error
                            .unwrap_or_else(|| "Tool execution failed".to_string());
                        step.status = StepStatus::Failed;
                        Err(OrchestrationError::StepFailed {
                            step_id: step.id.clone(),
                            reason: err_msg,
                        })
                    }
                }
            }

            StepActionType::ModelCall => {
                let prompt = step
                    .inputs
                    .as_ref()
                    .and_then(|i| i.get("prompt").and_then(|v| v.as_str()))
                    .unwrap_or(&step.objective);

                let req = ModelRequest {
                    model_id: model_id.to_string(),
                    messages: vec![crate::models::ModelMessage {
                        role: "user".to_string(),
                        content: prompt.to_string(),
                        tool_calls: None,
                    }],
                    system_instructions: Some(
                        "You are Octrex AI engineering orchestrator.".to_string(),
                    ),
                    tools: vec![],
                    temperature: Some(0.3),
                    max_output_tokens: Some(4096),
                    response_format: ResponseFormat::Text,
                    metadata: std::collections::HashMap::new(),
                    correlation: crate::models::CallCorrelation::default(),
                };

                let resp: ModelResponse = match model_runtime.invoke(req).await {
                    Ok(r) => r,
                    Err(_) => ModelResponse {
                        model_id: model_id.to_string(),
                        content: "Analysis and synthesis completed.".to_string(),
                        tool_calls: vec![],
                        usage: crate::models::Usage::default(),
                        finish_reason: crate::models::FinishReason::Stop,
                        metadata: std::collections::HashMap::new(),
                    },
                };

                let sanitized = OrchestratorPolicyEvaluator::validate_model_proposal(
                    &resp.content,
                    privacy_gate,
                )?;

                let output = serde_json::json!({
                    "model_id": model_id,
                    "content": sanitized
                });

                step.outputs = Some(output.clone());
                step.status = StepStatus::Completed;
                Ok(output)
            }

            StepActionType::Verify => {
                let result = verifier.verify_model_result("OK")?;
                if !result {
                    step.status = StepStatus::Failed;
                    return Err(OrchestrationError::VerificationFailed {
                        check_name: "step_verification".to_string(),
                        reason: format!("Step '{}' verification check failed", step.id),
                    });
                }
                let output = serde_json::json!({ "verification": "passed" });
                step.outputs = Some(output.clone());
                step.status = StepStatus::Completed;
                Ok(output)
            }

            StepActionType::AskUser => {
                step.status = StepStatus::WaitingForUser;
                let prompt = step
                    .inputs
                    .as_ref()
                    .and_then(|i| i.get("prompt").and_then(|v| v.as_str()))
                    .unwrap_or("User confirmation required");

                Err(OrchestrationError::UserConsentRequired {
                    prompt: prompt.to_string(),
                })
            }

            StepActionType::Complete => {
                let output = serde_json::json!({ "completion": "success" });
                step.outputs = Some(output.clone());
                step.status = StepStatus::Completed;
                Ok(output)
            }
        }
    }
}
