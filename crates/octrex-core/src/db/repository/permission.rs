use crate::db::manager::DatabaseManager;
use crate::db::models::{PermissionDecision, PermissionDuration, PermissionRecord};
use crate::error::OctrexError;
use crate::ids::WorkspaceId;
use rusqlite::params;
use std::str::FromStr;

pub trait PermissionRepository: Send + Sync {
    fn create_permission(&self, perm: &PermissionRecord) -> Result<PermissionRecord, OctrexError>;
    fn get_permission(&self, id: &str) -> Result<Option<PermissionRecord>, OctrexError>;
    fn list_permissions(&self) -> Result<Vec<PermissionRecord>, OctrexError>;
    fn update_permission(&self, perm: &PermissionRecord) -> Result<PermissionRecord, OctrexError>;
    fn delete_permission(&self, id: &str) -> Result<bool, OctrexError>;
}

pub struct SqlitePermissionRepository {
    db: DatabaseManager,
}

impl SqlitePermissionRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
}

impl PermissionRepository for SqlitePermissionRepository {
    fn create_permission(&self, perm: &PermissionRecord) -> Result<PermissionRecord, OctrexError> {
        self.db.with_conn(|conn| {
            let ws_id_str = perm.workspace_id.as_ref().map(|w| w.as_str());
            conn.execute(
                "INSERT INTO permissions (id, scope, resource, decision, duration, workspace_id, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    perm.id,
                    perm.scope,
                    perm.resource,
                    perm.decision.to_string(),
                    perm.duration.to_string(),
                    ws_id_str,
                    perm.created_at,
                    perm.updated_at
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to insert permission: {}", e),
            })?;
            Ok(perm.clone())
        })
    }

    fn get_permission(&self, id: &str) -> Result<Option<PermissionRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, scope, resource, decision, duration, workspace_id, created_at, updated_at FROM permissions WHERE id = ?1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let result = stmt.query_row(params![id], |row| {
                let id_str: String = row.get(0)?;
                let scope: String = row.get(1)?;
                let resource: String = row.get(2)?;
                let decision_str: String = row.get(3)?;
                let duration_str: String = row.get(4)?;
                let ws_id_str: Option<String> = row.get(5)?;
                let created_at: u64 = row.get(6)?;
                let updated_at: u64 = row.get(7)?;

                let decision = PermissionDecision::from_str(&decision_str).unwrap_or(PermissionDecision::Ask);
                let duration = PermissionDuration::from_str(&duration_str).unwrap_or(PermissionDuration::Once);

                Ok(PermissionRecord {
                    id: id_str,
                    scope,
                    resource,
                    decision,
                    duration,
                    workspace_id: ws_id_str.map(WorkspaceId::from),
                    created_at,
                    updated_at,
                })
            });

            match result {
                Ok(p) => Ok(Some(p)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(OctrexError::Internal { message: e.to_string() }),
            }
        })
    }

    fn list_permissions(&self) -> Result<Vec<PermissionRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, scope, resource, decision, duration, workspace_id, created_at, updated_at FROM permissions ORDER BY updated_at DESC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map([], |row| {
                    let id_str: String = row.get(0)?;
                    let scope: String = row.get(1)?;
                    let resource: String = row.get(2)?;
                    let decision_str: String = row.get(3)?;
                    let duration_str: String = row.get(4)?;
                    let ws_id_str: Option<String> = row.get(5)?;
                    let created_at: u64 = row.get(6)?;
                    let updated_at: u64 = row.get(7)?;

                    let decision = PermissionDecision::from_str(&decision_str).unwrap_or(PermissionDecision::Ask);
                    let duration = PermissionDuration::from_str(&duration_str).unwrap_or(PermissionDuration::Once);

                    Ok(PermissionRecord {
                        id: id_str,
                        scope,
                        resource,
                        decision,
                        duration,
                        workspace_id: ws_id_str.map(WorkspaceId::from),
                        created_at,
                        updated_at,
                    })
                })
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let mut list = Vec::new();
            for r in rows {
                list.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
            }
            Ok(list)
        })
    }

    fn update_permission(&self, perm: &PermissionRecord) -> Result<PermissionRecord, OctrexError> {
        self.db.with_conn(|conn| {
            let ws_id_str = perm.workspace_id.as_ref().map(|w| w.as_str());
            let count = conn
                .execute(
                    "UPDATE permissions SET scope = ?1, resource = ?2, decision = ?3, duration = ?4, workspace_id = ?5, updated_at = ?6 WHERE id = ?7",
                    params![
                        perm.scope,
                        perm.resource,
                        perm.decision.to_string(),
                        perm.duration.to_string(),
                        ws_id_str,
                        perm.updated_at,
                        perm.id
                    ],
                )
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            if count == 0 {
                Err(OctrexError::NotFound {
                    resource: format!("Permission record with id '{}'", perm.id),
                })
            } else {
                Ok(perm.clone())
            }
        })
    }

    fn delete_permission(&self, id: &str) -> Result<bool, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute("DELETE FROM permissions WHERE id = ?1", params![id])
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;
            Ok(count > 0)
        })
    }
}
