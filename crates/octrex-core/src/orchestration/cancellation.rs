use crate::ids::TaskId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

#[derive(Clone, Default)]
pub struct CancellationManager {
    tokens: Arc<RwLock<HashMap<TaskId, Arc<AtomicBool>>>>,
}

impl CancellationManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_task(&self, task_id: &TaskId) -> Arc<AtomicBool> {
        let mut guard = self.tokens.write().unwrap();
        let flag = Arc::new(AtomicBool::new(false));
        guard.insert(task_id.clone(), flag.clone());
        flag
    }

    pub fn cancel_task(&self, task_id: &TaskId) -> bool {
        let guard = self.tokens.read().unwrap();
        if let Some(flag) = guard.get(task_id) {
            flag.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn is_cancelled(&self, task_id: &TaskId) -> bool {
        let guard = self.tokens.read().unwrap();
        if let Some(flag) = guard.get(task_id) {
            flag.load(Ordering::SeqCst)
        } else {
            false
        }
    }

    pub fn remove(&self, task_id: &TaskId) {
        let mut guard = self.tokens.write().unwrap();
        guard.remove(task_id);
    }
}
