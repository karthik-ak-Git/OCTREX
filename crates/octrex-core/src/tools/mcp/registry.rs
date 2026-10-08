use crate::tools::mcp::types::McpServerInfo;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct McpRegistry {
    servers: RwLock<HashMap<String, McpServerInfo>>,
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self {
            servers: RwLock::new(HashMap::new()),
        }
    }
}

impl McpRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_server(&self, server: McpServerInfo) {
        let mut lock = self.servers.write().unwrap();
        lock.insert(server.server_id.clone(), server);
    }

    pub fn get_server(&self, server_id: &str) -> Option<McpServerInfo> {
        let lock = self.servers.read().unwrap();
        lock.get(server_id).cloned()
    }

    pub fn list_servers(&self) -> Vec<McpServerInfo> {
        let lock = self.servers.read().unwrap();
        let mut list: Vec<_> = lock.values().cloned().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn set_enabled(&self, server_id: &str, enabled: bool) -> bool {
        let mut lock = self.servers.write().unwrap();
        if let Some(srv) = lock.get_mut(server_id) {
            srv.enabled = enabled;
            srv.status = if enabled {
                "CONNECTED".to_string()
            } else {
                "DISABLED".to_string()
            };
            true
        } else {
            false
        }
    }

    pub fn format_namespaced_tool_id(server_id: &str, tool_name: &str) -> String {
        format!("mcp.{}.{}", server_id, tool_name)
    }
}
