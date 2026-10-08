use crate::db::manager::DatabaseManager;
use crate::db::models::EventRecord;
use crate::error::OctrexError;
use crate::ids::{SessionId, TaskId, WorkspaceId};
use rusqlite::params;

pub trait EventRepository: Send + Sync {
    fn record_event(&self, event: &EventRecord) -> Result<EventRecord, OctrexError>;
    fn get_event(&self, id: &str) -> Result<Option<EventRecord>, OctrexError>;
    fn list_events(&self, limit: usize) -> Result<Vec<EventRecord>, OctrexError>;
    fn list_events_by_type(
        &self,
        event_type: &str,
        limit: usize,
    ) -> Result<Vec<EventRecord>, OctrexError>;
}

pub struct SqliteEventRepository {
    db: DatabaseManager,
}

impl SqliteEventRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
}

impl EventRepository for SqliteEventRepository {
    fn record_event(&self, event: &EventRecord) -> Result<EventRecord, OctrexError> {
        self.db.with_conn(|conn| {
            let sess_id_str = event.session_id.as_ref().map(|s| s.as_str());
            let task_id_str = event.task_id.as_ref().map(|t| t.as_str());
            let ws_id_str = event.workspace_id.as_ref().map(|w| w.as_str());
            let payload_str = event.payload.to_string();

            conn.execute(
                "INSERT INTO events (id, event_type, payload, timestamp, session_id, task_id, workspace_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    event.id,
                    event.event_type,
                    payload_str,
                    event.timestamp,
                    sess_id_str,
                    task_id_str,
                    ws_id_str
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to record event: {}", e),
            })?;
            Ok(event.clone())
        })
    }

    fn get_event(&self, id: &str) -> Result<Option<EventRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, event_type, payload, timestamp, session_id, task_id, workspace_id FROM events WHERE id = ?1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let result = stmt.query_row(params![id], |row| {
                let id_str: String = row.get(0)?;
                let event_type: String = row.get(1)?;
                let payload_str: String = row.get(2)?;
                let timestamp: u64 = row.get(3)?;
                let sess_id_str: Option<String> = row.get(4)?;
                let task_id_str: Option<String> = row.get(5)?;
                let ws_id_str: Option<String> = row.get(6)?;

                let payload = serde_json::from_str(&payload_str).unwrap_or_default();

                Ok(EventRecord {
                    id: id_str,
                    event_type,
                    payload,
                    timestamp,
                    session_id: sess_id_str.map(SessionId::from),
                    task_id: task_id_str.map(TaskId::from),
                    workspace_id: ws_id_str.map(WorkspaceId::from),
                })
            });

            match result {
                Ok(ev) => Ok(Some(ev)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(OctrexError::Internal { message: e.to_string() }),
            }
        })
    }

    fn list_events(&self, limit: usize) -> Result<Vec<EventRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, event_type, payload, timestamp, session_id, task_id, workspace_id FROM events ORDER BY timestamp DESC LIMIT ?1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map(params![limit as i64], |row| {
                    let id_str: String = row.get(0)?;
                    let event_type: String = row.get(1)?;
                    let payload_str: String = row.get(2)?;
                    let timestamp: u64 = row.get(3)?;
                    let sess_id_str: Option<String> = row.get(4)?;
                    let task_id_str: Option<String> = row.get(5)?;
                    let ws_id_str: Option<String> = row.get(6)?;

                    let payload = serde_json::from_str(&payload_str).unwrap_or_default();

                    Ok(EventRecord {
                        id: id_str,
                        event_type,
                        payload,
                        timestamp,
                        session_id: sess_id_str.map(SessionId::from),
                        task_id: task_id_str.map(TaskId::from),
                        workspace_id: ws_id_str.map(WorkspaceId::from),
                    })
                })
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let mut events = Vec::new();
            for r in rows {
                events.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
            }
            Ok(events)
        })
    }

    fn list_events_by_type(
        &self,
        event_type: &str,
        limit: usize,
    ) -> Result<Vec<EventRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, event_type, payload, timestamp, session_id, task_id, workspace_id FROM events WHERE event_type = ?1 ORDER BY timestamp DESC LIMIT ?2")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map(params![event_type, limit as i64], |row| {
                    let id_str: String = row.get(0)?;
                    let event_type: String = row.get(1)?;
                    let payload_str: String = row.get(2)?;
                    let timestamp: u64 = row.get(3)?;
                    let sess_id_str: Option<String> = row.get(4)?;
                    let task_id_str: Option<String> = row.get(5)?;
                    let ws_id_str: Option<String> = row.get(6)?;

                    let payload = serde_json::from_str(&payload_str).unwrap_or_default();

                    Ok(EventRecord {
                        id: id_str,
                        event_type,
                        payload,
                        timestamp,
                        session_id: sess_id_str.map(SessionId::from),
                        task_id: task_id_str.map(TaskId::from),
                        workspace_id: ws_id_str.map(WorkspaceId::from),
                    })
                })
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let mut events = Vec::new();
            for r in rows {
                events.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
            }
            Ok(events)
        })
    }
}
