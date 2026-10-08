use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionEngineBoundary {
    pub auto_approve_read: bool,
}

impl Default for PermissionEngineBoundary {
    fn default() -> Self {
        Self {
            auto_approve_read: true,
        }
    }
}
