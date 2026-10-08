use crate::tools::types::{ToolCapability, ToolDescriptor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum McpTrustLevel {
    Untrusted,
    Restricted,
    Trusted,
}

impl Default for McpTrustLevel {
    fn default() -> Self {
        McpTrustLevel::Untrusted
    }
}

impl std::fmt::Display for McpTrustLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpTrustLevel::Untrusted => write!(f, "UNTRUSTED"),
            McpTrustLevel::Restricted => write!(f, "RESTRICTED"),
            McpTrustLevel::Trusted => write!(f, "TRUSTED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub server_id: String,
    pub name: String,
    pub transport: String,
    pub endpoint: String,
    pub enabled: bool,
    pub trust_level: McpTrustLevel,
    pub declared_capabilities: Vec<ToolCapability>,
    pub allowed_tools: Vec<String>,
    pub status: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDescriptor {
    pub mcp_server_id: String,
    pub raw_tool_name: String,
    pub namespaced_tool_id: String, // format: mcp.<server_id>.<tool_name>
    pub descriptor: ToolDescriptor,
}
