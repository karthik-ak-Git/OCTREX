use crate::db::manager::DatabaseManager;
use crate::error::OctrexError;
use crate::ids::{SessionId, WorkspaceId};
use crate::session::{Session, SessionStatus};
use rusqlite::params;

pub trait SessionRepository: Send + Sync {
    fn create_session(&self, session: &Session) -> Result<Session, OctrexError>;
    fn get_session(&self, id: &SessionId) -> Result<Option<Session>, OctrexError>;
    fn list_sessions(&self) -> Result<Vec<Session>, OctrexError>;
    fn list_sessions_by_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<Session>, OctrexError>;
    fn update_session(&self, session: &Session) -> Result<Session, OctrexError>;
    fn delete_session(&self, id: &SessionId) -> Result<bool, OctrexError>;
}

pub struct SqliteSessionRepository {
    db: DatabaseManager,
}

impl SqliteSessionRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
}

impl SessionRepository for SqliteSessionRepository {
    fn create_session(&self, session: &Session) -> Result<Session, OctrexError> {
        self.db.with_conn(|conn| {
            let ws_id_str = session.workspace_id.as_ref().map(|w| w.as_str());
            let status_str = match session.status {
                SessionStatus::Active => "ACTIVE",
                SessionStatus::Archived => "ARCHIVED",
                SessionStatus::Closed => "CLOSED",
            };

            conn.execute(
                "INSERT INTO sessions (id, workspace_id, title, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    session.id.as_str(),
                    ws_id_str,
                    session.title,
                    status_str,
                    session.created_at,
                    session.updated_at
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to create session in database: {}", e),
            })?;
            Ok(session.clone())
        })
    }

    fn get_session(&self, id: &SessionId) -> Result<Option<Session>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, workspace_id, title, status, created_at, updated_at FROM sessions WHERE id = ?1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let result = stmt.query_row(params![id.as_str()], |row| {
                let id_str: String = row.get(0)?;
                let ws_id_str: Option<String> = row.get(1)?;
                let title: String = row.get(2)?;
                let status_str: String = row.get(3)?;
                let created_at: u64 = row.get(4)?;
                let updated_at: u64 = row.get(5)?;

                let status = match status_str.to_uppercase().as_str() {
                    "ARCHIVED" => SessionStatus::Archived,
                    "CLOSED" => SessionStatus::Closed,
                    _ => SessionStatus::Active,
                };

                Ok(Session {
                    id: SessionId::from(id_str),
                    workspace_id: ws_id_str.map(WorkspaceId::from),
                    title,
                    status,
                    created_at,
                    updated_at,
                })
            });

            match result {
                Ok(s) => Ok(Some(s)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(OctrexError::Internal { message: e.to_string() }),
            }
        })
    }

    fn list_sessions(&self) -> Result<Vec<Session>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, workspace_id, title, status, created_at, updated_at FROM sessions ORDER BY updated_at DESC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map([], |row| {
                    let id_str: String = row.get(0)?;
                    let ws_id_str: Option<String> = row.get(1)?;
                    let title: String = row.get(2)?;
                    let status_str: String = row.get(3)?;
                    let created_at: u64 = row.get(4)?;
                    let updated_at: u64 = row.get(5)?;

                    let status = match status_str.to_uppercase().as_str() {
                        "ARCHIVED" => SessionStatus::Archived,
                        "CLOSED" => SessionStatus::Closed,
                        _ => SessionStatus::Active,
                    };

                    Ok(Session {
                        id: SessionId::from(id_str),
                        workspace_id: ws_id_str.map(WorkspaceId::from),
                        title,
                        status,
                        created_at,
                        updated_at,
                    })
                })
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let mut sessions = Vec::new();
            for r in rows {
                sessions.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
            }
            Ok(sessions)
        })
    }

    fn list_sessions_by_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<Session>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, workspace_id, title, status, created_at, updated_at FROM sessions WHERE workspace_id = ?1 ORDER BY updated_at DESC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map(params![workspace_id.as_str()], |row| {
                    let id_str: String = row.get(0)?;
                    let ws_id_str: Option<String> = row.get(1)?;
                    let title: String = row.get(2)?;
                    let status_str: String = row.get(3)?;
                    let created_at: u64 = row.get(4)?;
                    let updated_at: u64 = row.get(5)?;

                    let status = match status_str.to_uppercase().as_str() {
                        "ARCHIVED" => SessionStatus::Archived,
                        "CLOSED" => SessionStatus::Closed,
                        _ => SessionStatus::Active,
                    };

                    Ok(Session {
                        id: SessionId::from(id_str),
                        workspace_id: ws_id_str.map(WorkspaceId::from),
                        title,
                        status,
                        created_at,
                        updated_at,
                    })
                })
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let mut sessions = Vec::new();
            for r in rows {
                sessions.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
            }
            Ok(sessions)
        })
    }

    fn update_session(&self, session: &Session) -> Result<Session, OctrexError> {
        self.db.with_conn(|conn| {
            let ws_id_str = session.workspace_id.as_ref().map(|w| w.as_str());
            let status_str = match session.status {
                SessionStatus::Active => "ACTIVE",
                SessionStatus::Archived => "ARCHIVED",
                SessionStatus::Closed => "CLOSED",
            };

            let count = conn
                .execute(
                    "UPDATE sessions SET workspace_id = ?1, title = ?2, status = ?3, updated_at = ?4 WHERE id = ?5",
                    params![
                        ws_id_str,
                        session.title,
                        status_str,
                        session.updated_at,
                        session.id.as_str()
                    ],
                )
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            if count == 0 {
                Err(OctrexError::NotFound {
                    resource: format!("Session with id '{}'", session.id),
                })
            } else {
                Ok(session.clone())
            }
        })
    }

    fn delete_session(&self, id: &SessionId) -> Result<bool, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute("DELETE FROM sessions WHERE id = ?1", params![id.as_str()])
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;
            Ok(count > 0)
        })
    }
}
