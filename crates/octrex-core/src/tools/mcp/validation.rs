use crate::privacy::EvidenceManager;
use crate::tools::validator::ToolValidator;
use serde_json::Value;

pub struct McpValidator;

impl McpValidator {
    pub fn sanitize_mcp_output(
        raw_output: Value,
        evidence_mgr: &EvidenceManager,
    ) -> (Value, Vec<String>) {
        let (sanitized, _status, mut warnings) =
            ToolValidator::sanitize_and_validate_output(raw_output, evidence_mgr);

        warnings.push("MCP response output treated as untrusted third-party data".to_string());
        (sanitized, warnings)
    }
}
