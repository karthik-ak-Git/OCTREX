use crate::db::manager::DatabaseManager;
use crate::events::{EventBus, EventEnvelope, EventType};
use crate::network::NetworkSecurityService;
use crate::privacy::{EvidenceManager, PrivacyGate};
use crate::tools::audit::ToolAuditLogger;
use crate::tools::capability::CapabilityGrant;
use crate::tools::consent::ToolConsentManager;
use crate::tools::executor::BuiltInToolExecutor;
use crate::tools::mcp::McpRegistry;
use crate::tools::policy::ToolPolicyEvaluator;
use crate::tools::registry::ToolRegistry;
use crate::tools::request::ToolRequest;
use crate::tools::response::{ToolDecision, ToolResponse};
use crate::tools::sandbox::ToolExecutionContext;
use crate::tools::types::CapabilityScope;
use crate::tools::validator::ToolValidator;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};
use tokio::time::timeout;

pub const MAX_RECURSION_DEPTH: u32 = 5;
pub const MAX_CONCURRENT_TOOL_EXECUTIONS: u32 = 10;

pub struct ToolRuntime {
    registry: Arc<ToolRegistry>,
    mcp_registry: Arc<McpRegistry>,
    consent_manager: Arc<ToolConsentManager>,
    evidence_manager: Arc<EvidenceManager>,
    event_bus: Arc<EventBus>,
    db: Arc<DatabaseManager>,
    network_security: Arc<NetworkSecurityService>,
    privacy_gate: Arc<PrivacyGate>,
    active_executions: AtomicU32,
    grants: RwLock<HashMap<String, CapabilityGrant>>,
}

impl ToolRuntime {
    pub fn new(
        registry: Arc<ToolRegistry>,
        mcp_registry: Arc<McpRegistry>,
        event_bus: Arc<EventBus>,
        db: Arc<DatabaseManager>,
        network_security: Arc<NetworkSecurityService>,
        privacy_gate: Arc<PrivacyGate>,
    ) -> Self {
        Self {
            registry,
            mcp_registry,
            consent_manager: Arc::new(ToolConsentManager::new()),
            evidence_manager: Arc::new(EvidenceManager),
            event_bus,
            db,
            network_security,
            privacy_gate,
            active_executions: AtomicU32::new(0),
            grants: RwLock::new(HashMap::new()),
        }
    }

    pub fn registry(&self) -> &Arc<ToolRegistry> {
        &self.registry
    }

    pub fn mcp_registry(&self) -> &Arc<McpRegistry> {
        &self.mcp_registry
    }

    pub fn consent_manager(&self) -> &Arc<ToolConsentManager> {
        &self.consent_manager
    }

    pub fn evaluate_request(&self, req: &ToolRequest) -> ToolDecision {
        // Look up descriptor
        let descriptor = match self.registry.get(&req.tool_id) {
            Some(d) => d,
            None => {
                // Check if it's an MCP tool reference
                if req.tool_id.as_str().starts_with("mcp.") {
                    let parts: Vec<&str> = req.tool_id.as_str().splitn(3, '.').collect();
                    if parts.len() == 3 {
                        if let Some(mcp_server) = self.mcp_registry.get_server(parts[1]) {
                            if !mcp_server.enabled {
                                return ToolDecision::block(
                                    format!("MCP Server '{}' is disabled", parts[1]),
                                    "mcp_security_boundary",
                                );
                            }
                            return ToolDecision::allow(
                                format!("MCP tool '{}' authorized", req.tool_id),
                                mcp_server.declared_capabilities,
                                crate::tools::types::RiskLevel::Medium,
                            );
                        }
                    }
                }
                return ToolDecision::block(
                    format!("Tool '{}' not found in tool registry", req.tool_id),
                    "tool_registry",
                );
            }
        };

        match ToolPolicyEvaluator::evaluate(
            req,
            &descriptor,
            &self.privacy_gate,
            &self.network_security,
        ) {
            Ok(decision) => decision,
            Err(e) => ToolDecision::block(e.to_string(), "tool_policy_evaluator"),
        }
    }

    pub async fn execute_tool(
        &self,
        req: ToolRequest,
        workspace_path: Option<PathBuf>,
    ) -> ToolResponse {
        let call_id = format!("call-{}", uuid::Uuid::new_v4().simple());
        let start_time = Instant::now();

        // Step 19: Recursion limit check
        if req.recursion_depth > MAX_RECURSION_DEPTH {
            let resp = ToolResponse::blocked(
                &call_id,
                req.tool_id.clone(),
                format!(
                    "Tool recursion limit exceeded (depth {})",
                    req.recursion_depth
                ),
            );
            self.emit_event(
                EventType::ToolFailed,
                serde_json::json!({
                    "tool_id": req.tool_id.as_str(),
                    "error": "Recursion limit exceeded"
                }),
            );
            return resp;
        }

        // Concurrency limit check
        let current_concurrency = self.active_executions.fetch_add(1, Ordering::SeqCst);
        if current_concurrency >= MAX_CONCURRENT_TOOL_EXECUTIONS {
            self.active_executions.fetch_sub(1, Ordering::SeqCst);
            let resp = ToolResponse::blocked(
                &call_id,
                req.tool_id.clone(),
                format!(
                    "Max concurrent tool executions limit ({}) reached",
                    MAX_CONCURRENT_TOOL_EXECUTIONS
                ),
            );
            return resp;
        }

        // Step 2 & 3: Look up tool descriptor
        let descriptor = match self.registry.get(&req.tool_id) {
            Some(d) => d,
            None => {
                self.active_executions.fetch_sub(1, Ordering::SeqCst);
                let resp = ToolResponse::blocked(
                    &call_id,
                    req.tool_id.clone(),
                    format!(
                        "Tool '{}' is unknown and blocked by security boundary",
                        req.tool_id
                    ),
                );
                self.audit_and_emit_blocked(&req, &call_id, "Tool not found in registry");
                return resp;
            }
        };

        // Step 4 & 5: Validate tool enabled state and input schema
        if !descriptor.enabled {
            self.active_executions.fetch_sub(1, Ordering::SeqCst);
            let resp = ToolResponse::blocked(
                &call_id,
                req.tool_id.clone(),
                format!("Tool '{}' is disabled", req.tool_id),
            );
            self.audit_and_emit_blocked(&req, &call_id, "Tool disabled");
            return resp;
        }

        if let Err(e) = ToolValidator::validate_input(&descriptor, &req.arguments) {
            self.active_executions.fetch_sub(1, Ordering::SeqCst);
            let resp = ToolResponse::blocked(&call_id, req.tool_id.clone(), e.to_string());
            self.audit_and_emit_blocked(&req, &call_id, &e.to_string());
            return resp;
        }

        // Step 6 - 14: Evaluate security policies (Policy, Privacy, Filesystem, Network)
        let decision = self.evaluate_request(&req);
        if !decision.is_allowed() {
            self.active_executions.fetch_sub(1, Ordering::SeqCst);
            let resp = ToolResponse::blocked(&call_id, req.tool_id.clone(), &decision.reason);
            self.audit_and_emit_blocked(&req, &call_id, &decision.reason);
            return resp;
        }

        // Step 15: Create scoped capability grant
        let grant_id = format!("grant-{}", uuid::Uuid::new_v4().simple());
        let now_sec = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let grant = CapabilityGrant {
            grant_id: grant_id.clone(),
            tool_id: req.tool_id.clone(),
            task_id: req.task_id.clone(),
            session_id: req.session_id.clone(),
            workspace_id: req.workspace_id.clone(),
            capabilities: decision.granted_capabilities.clone(),
            scope: CapabilityScope {
                workspace_id: req.workspace_id.clone(),
                allowed_paths: vec![],
                allowed_domains: vec![],
                allowed_commands: vec![],
            },
            expiration: Some(now_sec + 300), // 5 minute grant
            policy_source: decision.policy_source.clone(),
            consent: true,
            created_at: now_sec,
        };

        {
            let mut lock = self.grants.write().unwrap();
            lock.insert(grant_id.clone(), grant.clone());
        }

        // Build controlled sandbox context
        let sandbox_ctx =
            ToolExecutionContext::from_grant(&grant, workspace_path, descriptor.timeout_ms);

        // Emit TOOL_STARTED event
        self.emit_event(
            EventType::ToolStarted,
            serde_json::json!({
                "tool_id": req.tool_id.as_str(),
                "call_id": call_id,
                "arguments": req.arguments
            }),
        );

        // Step 16: Execute tool with timeout
        let exec_timeout = Duration::from_millis(descriptor.timeout_ms.max(1000));
        let tool_id_clone = req.tool_id.clone();
        let args_clone = req.arguments.clone();

        let raw_result = timeout(exec_timeout, async move {
            BuiltInToolExecutor::execute(&tool_id_clone, &args_clone, &sandbox_ctx).await
        })
        .await;

        self.active_executions.fetch_sub(1, Ordering::SeqCst);
        let duration_ms = start_time.elapsed().as_millis();

        match raw_result {
            Err(_) => {
                let resp = ToolResponse::failed(
                    &call_id,
                    req.tool_id.clone(),
                    format!("Tool execution timed out after {}ms", descriptor.timeout_ms),
                    duration_ms,
                );
                ToolAuditLogger::log_tool_execution(
                    &self.db,
                    &req.tool_id,
                    req.task_id.map(|t| t.to_string()),
                    req.session_id.map(|s| s.to_string()),
                    req.workspace_id.map(|w| w.to_string()),
                    &decision.policy_source,
                    &resp,
                );
                self.emit_event(
                    EventType::ToolFailed,
                    serde_json::json!({ "tool_id": req.tool_id.as_str(), "error": "Timeout" }),
                );
                resp
            }
            Ok(Err(e)) => {
                let resp =
                    ToolResponse::failed(&call_id, req.tool_id.clone(), e.to_string(), duration_ms);
                ToolAuditLogger::log_tool_execution(
                    &self.db,
                    &req.tool_id,
                    req.task_id.map(|t| t.to_string()),
                    req.session_id.map(|s| s.to_string()),
                    req.workspace_id.map(|w| w.to_string()),
                    &decision.policy_source,
                    &resp,
                );
                self.emit_event(
                    EventType::ToolFailed,
                    serde_json::json!({ "tool_id": req.tool_id.as_str(), "error": e.to_string() }),
                );
                resp
            }
            Ok(Ok(val)) => {
                // Step 17 & 18: Validate & sanitize untrusted output
                let (sanitized_val, status, warnings) =
                    ToolValidator::sanitize_and_validate_output(val, &self.evidence_manager);

                let mut resp = ToolResponse::success(
                    &call_id,
                    req.tool_id.clone(),
                    sanitized_val,
                    duration_ms,
                );
                resp.status = status;
                resp.warnings = warnings;

                // Audit & emit completed event
                ToolAuditLogger::log_tool_execution(
                    &self.db,
                    &req.tool_id,
                    req.task_id.map(|t| t.to_string()),
                    req.session_id.map(|s| s.to_string()),
                    req.workspace_id.map(|w| w.to_string()),
                    &decision.policy_source,
                    &resp,
                );
                self.emit_event(
                    EventType::ToolCompleted,
                    serde_json::json!({
                        "tool_id": req.tool_id.as_str(),
                        "call_id": call_id,
                        "duration_ms": duration_ms
                    }),
                );
                resp
            }
        }
    }

    fn audit_and_emit_blocked(&self, req: &ToolRequest, call_id: &str, reason: &str) {
        let resp = ToolResponse::blocked(call_id, req.tool_id.clone(), reason);
        ToolAuditLogger::log_tool_execution(
            &self.db,
            &req.tool_id,
            req.task_id.as_ref().map(|t| t.to_string()),
            req.session_id.as_ref().map(|s| s.to_string()),
            req.workspace_id.as_ref().map(|w| w.to_string()),
            "security_boundary",
            &resp,
        );
        self.emit_event(
            EventType::PermissionDenied,
            serde_json::json!({
                "tool_id": req.tool_id.as_str(),
                "call_id": call_id,
                "reason": reason
            }),
        );
    }

    fn emit_event(&self, event_type: EventType, payload: serde_json::Value) {
        let envelope = EventEnvelope::new(event_type, payload);
        let _ = self.event_bus.publish(envelope);
    }
}
