use super::types::NetworkCapability;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilityDeclaration {
    pub tool_name: String,
    pub declared_capabilities: HashSet<NetworkCapability>,
    pub max_execution_mode: String,
}

impl ToolCapabilityDeclaration {
    pub fn new(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            declared_capabilities: HashSet::new(),
            max_execution_mode: "online_allowed".to_string(),
        }
    }

    pub fn with_capability(mut self, capability: NetworkCapability) -> Self {
        self.declared_capabilities.insert(capability);
        self
    }

    pub fn supports_capability(&self, capability: NetworkCapability) -> bool {
        self.declared_capabilities.contains(&capability)
    }
}
