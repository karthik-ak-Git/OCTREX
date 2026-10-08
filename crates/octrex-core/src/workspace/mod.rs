pub mod manager;
pub mod types;

pub use manager::WorkspaceManager;
pub use types::*;

use crate::ids::WorkspaceId;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct WorkspaceRegistry {
    workspaces: Arc<RwLock<HashMap<WorkspaceId, Workspace>>>,
}

impl WorkspaceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_workspace(&self, name: impl Into<String>, path: PathBuf) -> Workspace {
        let ws = Workspace::new(name, path);
        let mut guard = self.workspaces.write().unwrap();
        guard.insert(ws.id.clone(), ws.clone());
        ws
    }

    pub fn get_workspace(&self, id: &WorkspaceId) -> Option<Workspace> {
        let guard = self.workspaces.read().unwrap();
        guard.get(id).cloned()
    }

    pub fn list_workspaces(&self) -> Vec<Workspace> {
        let guard = self.workspaces.read().unwrap();
        guard.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_registry() {
        let registry = WorkspaceRegistry::new();
        let ws = registry.register_workspace("OCTREX Project", PathBuf::from("d:\\OCTREX"));
        assert_eq!(ws.name, "OCTREX Project");

        let found = registry.get_workspace(&ws.id);
        assert!(found.is_some());
    }
}
