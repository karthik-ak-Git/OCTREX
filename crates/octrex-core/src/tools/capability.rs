use crate::ids::{SessionId, TaskId, WorkspaceId};
use crate::tools::types::{CapabilityScope, ToolCapability, ToolId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub grant_id: String,
    pub tool_id: ToolId,
    pub task_id: Option<TaskId>,
    pub session_id: Option<SessionId>,
    pub workspace_id: Option<WorkspaceId>,
    pub capabilities: Vec<ToolCapability>,
    pub scope: CapabilityScope,
    pub expiration: Option<u64>,
    pub policy_source: String,
    pub consent: bool,
    pub created_at: u64,
}

impl CapabilityGrant {
    pub fn is_expired(&self, current_time: u64) -> bool {
        if let Some(exp) = self.expiration {
            current_time >= exp
        } else {
            false
        }
    }

    pub fn grants_capability(&self, cap: &ToolCapability) -> bool {
        self.capabilities.contains(cap)
    }

    pub fn satisfies_scope(
        &self,
        req_workspace: Option<&WorkspaceId>,
        req_domain: Option<&str>,
    ) -> bool {
        if let Some(ref granted_ws) = self.scope.workspace_id {
            if let Some(req_ws) = req_workspace {
                if granted_ws != req_ws {
                    return false;
                }
            }
        }

        if let Some(domain) = req_domain {
            if !self.scope.allowed_domains.is_empty() {
                let allowed = self.scope.allowed_domains.iter().any(|d| {
                    if d == "*" || d == domain {
                        true
                    } else if d.starts_with("*.") {
                        let suffix = &d[1..];
                        domain.ends_with(suffix)
                    } else {
                        false
                    }
                });
                if !allowed {
                    return false;
                }
            }
        }

        true
    }
}
