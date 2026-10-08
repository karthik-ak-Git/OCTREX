use crate::tools::errors::ToolError;
use crate::tools::types::{RiskLevel, ToolCapability, ToolDescriptor, ToolId, ToolSource};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct ToolRegistry {
    tools: RwLock<HashMap<ToolId, ToolDescriptor>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        let registry = Self {
            tools: RwLock::new(HashMap::new()),
        };
        registry.register_builtins();
        registry
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    fn register_builtins(&self) {
        let builtins = vec![
            ToolDescriptor {
                id: ToolId::new("workspace_list"),
                name: "Workspace File List".to_string(),
                version: "1.0.0".to_string(),
                description: "Lists files and directories inside the active workspace".to_string(),
                source: ToolSource::BuiltIn,
                capabilities: vec![
                    ToolCapability::FilesystemList,
                    ToolCapability::WorkspaceRead,
                ],
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "subpath": { "type": "string" }
                    }
                }),
                output_schema: None,
                risk_level: RiskLevel::Low,
                enabled: true,
                requires_confirmation: false,
                timeout_ms: 10000,
                metadata: HashMap::new(),
            },
            ToolDescriptor {
                id: ToolId::new("workspace_read"),
                name: "Workspace File Reader".to_string(),
                version: "1.0.0".to_string(),
                description: "Reads file content from the active workspace".to_string(),
                source: ToolSource::BuiltIn,
                capabilities: vec![
                    ToolCapability::FilesystemRead,
                    ToolCapability::WorkspaceRead,
                ],
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    },
                    "required": ["path"]
                }),
                output_schema: None,
                risk_level: RiskLevel::Low,
                enabled: true,
                requires_confirmation: false,
                timeout_ms: 15000,
                metadata: HashMap::new(),
            },
            ToolDescriptor {
                id: ToolId::new("workspace_write"),
                name: "Workspace File Writer".to_string(),
                version: "1.0.0".to_string(),
                description: "Writes content to a file inside the active workspace".to_string(),
                source: ToolSource::BuiltIn,
                capabilities: vec![
                    ToolCapability::FilesystemWrite,
                    ToolCapability::WorkspaceWrite,
                ],
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["path", "content"]
                }),
                output_schema: None,
                risk_level: RiskLevel::Medium,
                enabled: true,
                requires_confirmation: false,
                timeout_ms: 15000,
                metadata: HashMap::new(),
            },
            ToolDescriptor {
                id: ToolId::new("workspace_search"),
                name: "Workspace Code Search".to_string(),
                version: "1.0.0".to_string(),
                description: "Searches for matching text in workspace files".to_string(),
                source: ToolSource::BuiltIn,
                capabilities: vec![
                    ToolCapability::FilesystemRead,
                    ToolCapability::WorkspaceRead,
                ],
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    },
                    "required": ["query"]
                }),
                output_schema: None,
                risk_level: RiskLevel::Low,
                enabled: true,
                requires_confirmation: false,
                timeout_ms: 20000,
                metadata: HashMap::new(),
            },
            ToolDescriptor {
                id: ToolId::new("system_info"),
                name: "System Spec Information".to_string(),
                version: "1.0.0".to_string(),
                description: "Retrieves basic non-sensitive system hardware/OS information"
                    .to_string(),
                source: ToolSource::BuiltIn,
                capabilities: vec![ToolCapability::SystemInfo],
                input_schema: serde_json::json!({ "type": "object" }),
                output_schema: None,
                risk_level: RiskLevel::Low,
                enabled: true,
                requires_confirmation: false,
                timeout_ms: 5000,
                metadata: HashMap::new(),
            },
        ];

        let mut lock = self.tools.write().unwrap();
        for b in builtins {
            lock.insert(b.id.clone(), b);
        }
        for d in crate::documents::tools::document_tool_descriptors() {
            lock.insert(d.id.clone(), d);
        }
    }

    pub fn register(&self, descriptor: ToolDescriptor) -> Result<(), ToolError> {
        let mut lock = self.tools.write().unwrap();
        lock.insert(descriptor.id.clone(), descriptor);
        Ok(())
    }

    pub fn unregister(&self, id: &ToolId) -> Result<bool, ToolError> {
        let mut lock = self.tools.write().unwrap();
        Ok(lock.remove(id).is_some())
    }

    pub fn get(&self, id: &ToolId) -> Option<ToolDescriptor> {
        let lock = self.tools.read().unwrap();
        lock.get(id).cloned()
    }

    pub fn list(&self) -> Vec<ToolDescriptor> {
        let lock = self.tools.read().unwrap();
        let mut list: Vec<_> = lock.values().cloned().collect();
        list.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
        list
    }

    pub fn enable(&self, id: &ToolId) -> Result<bool, ToolError> {
        let mut lock = self.tools.write().unwrap();
        if let Some(tool) = lock.get_mut(id) {
            tool.enabled = true;
            Ok(true)
        } else {
            Err(ToolError::ToolNotFound(id.as_str().to_string()))
        }
    }

    pub fn disable(&self, id: &ToolId) -> Result<bool, ToolError> {
        let mut lock = self.tools.write().unwrap();
        if let Some(tool) = lock.get_mut(id) {
            tool.enabled = false;
            Ok(true)
        } else {
            Err(ToolError::ToolNotFound(id.as_str().to_string()))
        }
    }
}
