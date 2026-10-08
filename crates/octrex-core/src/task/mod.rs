use crate::error::AppErrorResponse;
use crate::ids::{RequestId, SessionId, TaskId, WorkspaceId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    Created,
    Planning,
    PlanReady,
    Executing,
    WaitingForTool,
    WaitingForUser,
    Verifying,
    Retrying,
    Blocked,
    Running,
    Paused,
    Cancelled,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub request_id: Option<RequestId>,
    pub session_id: Option<SessionId>,
    pub workspace_id: Option<WorkspaceId>,
    pub title: String,
    pub status: TaskStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub error: Option<AppErrorResponse>,
}

impl Task {
    pub fn new(
        title: impl Into<String>,
        request_id: Option<RequestId>,
        session_id: Option<SessionId>,
        workspace_id: Option<WorkspaceId>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id: TaskId::new(),
            request_id,
            session_id,
            workspace_id,
            title: title.into(),
            status: TaskStatus::Created,
            created_at: now,
            updated_at: now,
            error: None,
        }
    }
}

#[derive(Clone, Default)]
pub struct TaskRegistry {
    tasks: Arc<RwLock<HashMap<TaskId, Task>>>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_task(
        &self,
        title: impl Into<String>,
        request_id: Option<RequestId>,
        session_id: Option<SessionId>,
        workspace_id: Option<WorkspaceId>,
    ) -> Task {
        let task = Task::new(title, request_id, session_id, workspace_id);
        let mut guard = self.tasks.write().unwrap();
        guard.insert(task.id.clone(), task.clone());
        task
    }

    pub fn get_task(&self, id: &TaskId) -> Option<Task> {
        let guard = self.tasks.read().unwrap();
        guard.get(id).cloned()
    }

    pub fn update_status(
        &self,
        id: &TaskId,
        status: TaskStatus,
        error: Option<AppErrorResponse>,
    ) -> Option<Task> {
        let mut guard = self.tasks.write().unwrap();
        if let Some(task) = guard.get_mut(id) {
            task.status = status;
            task.error = error;
            task.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            Some(task.clone())
        } else {
            None
        }
    }

    pub fn list_tasks(&self) -> Vec<Task> {
        let guard = self.tasks.read().unwrap();
        guard.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_registry_lifecycle() {
        let registry = TaskRegistry::new();
        let task = registry.create_task("Test Task", None, None, None);
        assert_eq!(task.status, TaskStatus::Created);

        let updated = registry.update_status(&task.id, TaskStatus::Running, None);
        assert!(updated.is_some());
        assert_eq!(updated.unwrap().status, TaskStatus::Running);

        let list = registry.list_tasks();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, task.id);
    }
}
