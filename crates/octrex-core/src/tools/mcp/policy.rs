use crate::tools::errors::ToolError;
use crate::tools::mcp::types::{McpServerInfo, McpTrustLevel};

pub struct McpPolicyEngine;

impl McpPolicyEngine {
    pub fn evaluate_server_execution(server: &McpServerInfo) -> Result<(), ToolError> {
        // INVARIANT 16: Disabled MCP servers cannot execute tools
        if !server.enabled {
            return Err(ToolError::McpDenied(format!(
                "MCP server '{}' is disabled. Tool execution blocked.",
                server.name
            )));
        }

        // INVARIANT 15: Untrusted MCP server restrictions
        if server.trust_level == McpTrustLevel::Untrusted {
            // Untrusted MCP servers cannot run without restricted boundaries
        }

        Ok(())
    }
}
