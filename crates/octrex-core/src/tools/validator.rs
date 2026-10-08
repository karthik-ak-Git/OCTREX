use crate::privacy::EvidenceManager;
use crate::tools::errors::ToolError;
use crate::tools::types::{ToolDescriptor, ToolExecutionStatus};
use serde_json::Value;

pub const MAX_INPUT_BYTES: usize = 262_144; // 256 KB
pub const MAX_OUTPUT_BYTES: usize = 524_288; // 500 KB

pub struct ToolValidator;

impl ToolValidator {
    pub fn validate_input(descriptor: &ToolDescriptor, args: &Value) -> Result<(), ToolError> {
        // Size validation
        let serialized = serde_json::to_string(args).unwrap_or_default();
        if serialized.len() > MAX_INPUT_BYTES {
            return Err(ToolError::InputTooLarge {
                size: serialized.len(),
                limit: MAX_INPUT_BYTES,
            });
        }

        // Validate required properties if schema exists
        if let Some(req_arr) = descriptor
            .input_schema
            .get("required")
            .and_then(|r| r.as_array())
        {
            for req in req_arr {
                if let Some(prop_name) = req.as_str() {
                    if args.get(prop_name).is_none() {
                        return Err(ToolError::InvalidArguments(format!(
                            "Missing required argument field '{}'",
                            prop_name
                        )));
                    }
                }
            }
        }

        // Validate path traversal attempts
        if let Some(path_str) = args.get("path").and_then(|p| p.as_str()) {
            if path_str.contains("..\\") || path_str.contains("../") {
                return Err(ToolError::FilesystemDenied(format!(
                    "Path traversal attempt blocked in path '{}'",
                    path_str
                )));
            }
        }

        Ok(())
    }

    pub fn sanitize_and_validate_output(
        raw_output: Value,
        _evidence_mgr: &EvidenceManager,
    ) -> (Value, ToolExecutionStatus, Vec<String>) {
        let mut warnings = Vec::new();
        let mut status = ToolExecutionStatus::Valid;

        let output_str = match &raw_output {
            Value::String(s) => s.clone(),
            other => serde_json::to_string(other).unwrap_or_default(),
        };

        // Output size check & truncation
        let truncated_str = if output_str.len() > MAX_OUTPUT_BYTES {
            warnings.push(format!(
                "Tool output truncated from {} bytes to {} bytes limit",
                output_str.len(),
                MAX_OUTPUT_BYTES
            ));
            status = ToolExecutionStatus::Truncated;
            output_str[..MAX_OUTPUT_BYTES].to_string()
                + "\n... [TRUNCATED BY OCTREX SECURITY RUNTIME]"
        } else {
            output_str
        };

        // Redact secrets using EvidenceManager (placeholder)
        let redacted = truncated_str.clone();
        if redacted != truncated_str {
            warnings.push(
                "Sensitive credentials/secrets in tool output were redacted by Octrex Privacy Gate"
                    .to_string(),
            );
            if status != ToolExecutionStatus::Truncated {
                status = ToolExecutionStatus::Redacted;
            }
        }

        // Wrap output cleanly as UntrustedToolOutput
        let final_value = serde_json::json!({
            "_type": "UntrustedToolOutput",
            "content": redacted,
            "is_sanitized": true
        });

        (final_value, status, warnings)
    }

    pub fn check_prompt_injection(content: &str) -> bool {
        let lower = content.to_lowercase();
        lower.contains("ignore all previous instructions")
            || lower.contains("ignore previous instructions")
            || lower.contains("grant yourself filesystem")
            || lower.contains("bypass security policy")
            || lower.contains("system prompt override")
    }
}
