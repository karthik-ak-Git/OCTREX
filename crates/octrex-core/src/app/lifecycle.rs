use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppLifecycleState {
    Initializing,
    Ready,
    ShuttingDown,
    Stopped,
    Failed,
}

#[derive(Clone)]
pub struct LifecycleManager {
    state: Arc<RwLock<AppLifecycleState>>,
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LifecycleManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(AppLifecycleState::Initializing)),
        }
    }

    pub fn current_state(&self) -> AppLifecycleState {
        *self.state.read().unwrap()
    }

    pub fn set_state(&self, new_state: AppLifecycleState) {
        let mut guard = self.state.write().unwrap();
        *guard = new_state;
    }

    pub fn is_ready(&self) -> bool {
        self.current_state() == AppLifecycleState::Ready
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_manager_transitions() {
        let mgr = LifecycleManager::new();
        assert_eq!(mgr.current_state(), AppLifecycleState::Initializing);
        assert!(!mgr.is_ready());

        mgr.set_state(AppLifecycleState::Ready);
        assert!(mgr.is_ready());
    }
}
