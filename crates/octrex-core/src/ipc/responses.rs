use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationInfo {
    pub name: String,
    pub version: String,
    pub backend_version: String,
    pub protocol_version: String,
    pub environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    pub state: String,
    pub uptime_ms: u128,
    pub active_tasks: usize,
    pub active_sessions: usize,
    pub active_workspaces: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendHealth {
    pub status: String,
    pub version: String,
    pub runtime: String,
    pub initialized: bool,
    pub subsystems: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationSummary {
    pub active_provider: String,
    pub active_workspace: Option<String>,
    pub provider_count: usize,
    pub privacy_mode: String,
    pub enforcement_level: String,
}
