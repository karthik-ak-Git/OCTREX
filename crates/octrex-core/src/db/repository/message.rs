use crate::db::manager::DatabaseManager;
use crate::db::models::{Message, MessageRole};
use crate::error::OctrexError;
use crate::ids::SessionId;
use rusqlite::params;
use std::str::FromStr;

pub trait MessageRepository: Send + Sync {
    fn create_message(&self, message: &Message) -> Result<Message, OctrexError>;
    fn get_message(&self, id: &str) -> Result<Option<Message>, OctrexError>;
    fn list_messages_by_session(&self, session_id: &SessionId)
        -> Result<Vec<Message>, OctrexError>;
    fn delete_message(&self, id: &str) -> Result<bool, OctrexError>;
}

pub struct SqliteMessageRepository {
    db: DatabaseManager,
}

impl SqliteMessageRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
}

impl MessageRepository for SqliteMessageRepository {
    fn create_message(&self, message: &Message) -> Result<Message, OctrexError> {
        self.db.with_conn(|conn| {
            let metadata_str = message.metadata.as_ref().map(|v| v.to_string());
            conn.execute(
                "INSERT INTO messages (id, session_id, role, content, metadata, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    message.id,
                    message.session_id.as_str(),
                    message.role.to_string(),
                    message.content,
                    metadata_str,
                    message.created_at
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to insert message into database: {}", e),
            })?;
            Ok(message.clone())
        })
    }

    fn get_message(&self, id: &str) -> Result<Option<Message>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, session_id, role, content, metadata, created_at FROM messages WHERE id = ?1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let result = stmt.query_row(params![id], |row| {
                let id_str: String = row.get(0)?;
                let session_id_str: String = row.get(1)?;
                let role_str: String = row.get(2)?;
                let content: String = row.get(3)?;
                let metadata_str: Option<String> = row.get(4)?;
                let created_at: u64 = row.get(5)?;

                let role = MessageRole::from_str(&role_str)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))))?;

                let metadata = metadata_str
                    .and_then(|s| serde_json::from_str(&s).ok());

                Ok(Message {
                    id: id_str,
                    session_id: SessionId::from(session_id_str),
                    role,
                    content,
                    metadata,
                    created_at,
                })
            });

            match result {
                Ok(m) => Ok(Some(m)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(OctrexError::Internal { message: e.to_string() }),
            }
        })
    }

    fn list_messages_by_session(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<Message>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, session_id, role, content, metadata, created_at FROM messages WHERE session_id = ?1 ORDER BY created_at ASC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map(params![session_id.as_str()], |row| {
                    let id_str: String = row.get(0)?;
                    let session_id_str: String = row.get(1)?;
                    let role_str: String = row.get(2)?;
                    let content: String = row.get(3)?;
                    let metadata_str: Option<String> = row.get(4)?;
                    let created_at: u64 = row.get(5)?;

                    let role = MessageRole::from_str(&role_str)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))))?;

                    let metadata = metadata_str
                        .and_then(|s| serde_json::from_str(&s).ok());

                    Ok(Message {
                        id: id_str,
                        session_id: SessionId::from(session_id_str),
                        role,
                        content,
                        metadata,
                        created_at,
                    })
                })
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let mut messages = Vec::new();
            for r in rows {
                messages.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
            }
            Ok(messages)
        })
    }

    fn delete_message(&self, id: &str) -> Result<bool, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute("DELETE FROM messages WHERE id = ?1", params![id])
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;
            Ok(count > 0)
        })
    }
}
