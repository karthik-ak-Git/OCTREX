use crate::ids::{TaskId, WorkspaceId};
use crate::tools::types::{ToolCapability, ToolId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConsentRecord {
    pub consent_id: String,
    pub tool_id: ToolId,
    pub task_id: Option<TaskId>,
    pub workspace_id: Option<WorkspaceId>,
    pub granted_capabilities: Vec<ToolCapability>,
    pub granted: bool,
    pub reason: String,
    pub timestamp: u64,
}

pub struct ToolConsentManager {
    records: RwLock<HashMap<String, ToolConsentRecord>>,
}

impl Default for ToolConsentManager {
    fn default() -> Self {
        Self {
            records: RwLock::new(HashMap::new()),
        }
    }
}

impl ToolConsentManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_consent(
        &self,
        tool_id: ToolId,
        task_id: Option<TaskId>,
        workspace_id: Option<WorkspaceId>,
        granted_capabilities: Vec<ToolCapability>,
        granted: bool,
        reason: impl Into<String>,
    ) -> ToolConsentRecord {
        let consent_id = format!("consent-{}", uuid::Uuid::new_v4().simple());
        let record = ToolConsentRecord {
            consent_id: consent_id.clone(),
            tool_id,
            task_id,
            workspace_id,
            granted_capabilities,
            granted,
            reason: reason.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        let mut lock = self.records.write().unwrap();
        lock.insert(consent_id, record.clone());
        record
    }

    pub fn check_consent(
        &self,
        tool_id: &ToolId,
        task_id: Option<&TaskId>,
        workspace_id: Option<&WorkspaceId>,
    ) -> bool {
        let lock = self.records.read().unwrap();
        for record in lock.values() {
            if record.tool_id == *tool_id && record.granted {
                if let Some(t_id) = task_id {
                    if record.task_id.as_ref() == Some(t_id) {
                        return true;
                    }
                }
                if let Some(w_id) = workspace_id {
                    if record.workspace_id.as_ref() == Some(w_id) {
                        return true;
                    }
                }
            }
        }
        false
    }
}
