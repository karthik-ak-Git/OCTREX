use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityBoundary {
    pub enforcement_level: String,
}

impl Default for SecurityBoundary {
    fn default() -> Self {
        Self {
            enforcement_level: "standard".to_string(),
        }
    }
}
