use crate::ids::{SessionId, WorkspaceId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionStatus {
    Active,
    Archived,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub workspace_id: Option<WorkspaceId>,
    pub title: String,
    pub status: SessionStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Session {
    pub fn new(title: impl Into<String>, workspace_id: Option<WorkspaceId>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id: SessionId::new(),
            workspace_id,
            title: title.into(),
            status: SessionStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Clone, Default)]
pub struct SessionRegistry {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_session(
        &self,
        title: impl Into<String>,
        workspace_id: Option<WorkspaceId>,
    ) -> Session {
        let session = Session::new(title, workspace_id);
        let mut guard = self.sessions.write().unwrap();
        guard.insert(session.id.clone(), session.clone());
        session
    }

    pub fn get_session(&self, id: &SessionId) -> Option<Session> {
        let guard = self.sessions.read().unwrap();
        guard.get(id).cloned()
    }

    pub fn list_sessions(&self) -> Vec<Session> {
        let guard = self.sessions.read().unwrap();
        guard.values().cloned().collect()
    }

    pub fn close_session(&self, id: &SessionId) -> bool {
        let mut guard = self.sessions.write().unwrap();
        if let Some(session) = guard.get_mut(id) {
            session.status = SessionStatus::Closed;
            session.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_registry() {
        let registry = SessionRegistry::new();
        let session = registry.create_session("Chat Session 1", None);
        assert_eq!(session.status, SessionStatus::Active);

        let retrieved = registry.get_session(&session.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Chat Session 1");
    }
}
