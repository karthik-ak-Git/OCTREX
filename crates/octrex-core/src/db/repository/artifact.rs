use crate::db::manager::DatabaseManager;
use crate::db::models::ArtifactRecord;
use crate::error::OctrexError;
use crate::ids::{ArtifactId, TaskId, WorkspaceId};
use rusqlite::params;

pub trait ArtifactRepository: Send + Sync {
    fn create_artifact(&self, artifact: &ArtifactRecord) -> Result<ArtifactRecord, OctrexError>;
    fn get_artifact(&self, id: &ArtifactId) -> Result<Option<ArtifactRecord>, OctrexError>;
    fn list_artifacts_by_task(&self, task_id: &TaskId) -> Result<Vec<ArtifactRecord>, OctrexError>;
    fn list_artifacts_by_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<ArtifactRecord>, OctrexError>;
    fn update_verification_status(
        &self,
        id: &ArtifactId,
        status: &str,
    ) -> Result<bool, OctrexError>;
    fn delete_artifact(&self, id: &ArtifactId) -> Result<bool, OctrexError>;
}

pub struct SqliteArtifactRepository {
    db: DatabaseManager,
}

impl SqliteArtifactRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
}

impl ArtifactRepository for SqliteArtifactRepository {
    fn create_artifact(&self, artifact: &ArtifactRecord) -> Result<ArtifactRecord, OctrexError> {
        self.db.with_conn(|conn| {
            let task_id_str = artifact.task_id.as_ref().map(|t| t.as_str());
            let ws_id_str = artifact.workspace_id.as_ref().map(|w| w.as_str());

            conn.execute(
                "INSERT INTO artifacts (id, task_id, workspace_id, name, path, artifact_type, size, checksum, created_at, verification_status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    artifact.id.as_str(),
                    task_id_str,
                    ws_id_str,
                    artifact.name,
                    artifact.path,
                    artifact.artifact_type,
                    artifact.size,
                    artifact.checksum,
                    artifact.created_at,
                    artifact.verification_status
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to insert artifact record: {}", e),
            })?;
            Ok(artifact.clone())
        })
    }

    fn get_artifact(&self, id: &ArtifactId) -> Result<Option<ArtifactRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, task_id, workspace_id, name, path, artifact_type, size, checksum, created_at, verification_status FROM artifacts WHERE id = ?1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let result = stmt.query_row(params![id.as_str()], |row| {
                let id_str: String = row.get(0)?;
                let task_id_str: Option<String> = row.get(1)?;
                let ws_id_str: Option<String> = row.get(2)?;
                let name: String = row.get(3)?;
                let path: String = row.get(4)?;
                let artifact_type: String = row.get(5)?;
                let size: u64 = row.get(6)?;
                let checksum: Option<String> = row.get(7)?;
                let created_at: u64 = row.get(8)?;
                let verification_status: String = row.get(9)?;

                Ok(ArtifactRecord {
                    id: ArtifactId::from(id_str),
                    task_id: task_id_str.map(TaskId::from),
                    workspace_id: ws_id_str.map(WorkspaceId::from),
                    name,
                    path,
                    artifact_type,
                    size,
                    checksum,
                    created_at,
                    verification_status,
                })
            });

            match result {
                Ok(a) => Ok(Some(a)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(OctrexError::Internal { message: e.to_string() }),
            }
        })
    }

    fn list_artifacts_by_task(&self, task_id: &TaskId) -> Result<Vec<ArtifactRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, task_id, workspace_id, name, path, artifact_type, size, checksum, created_at, verification_status FROM artifacts WHERE task_id = ?1 ORDER BY created_at DESC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map(params![task_id.as_str()], |row| {
                    let id_str: String = row.get(0)?;
                    let task_id_str: Option<String> = row.get(1)?;
                    let ws_id_str: Option<String> = row.get(2)?;
                    let name: String = row.get(3)?;
                    let path: String = row.get(4)?;
                    let artifact_type: String = row.get(5)?;
                    let size: u64 = row.get(6)?;
                    let checksum: Option<String> = row.get(7)?;
                    let created_at: u64 = row.get(8)?;
                    let verification_status: String = row.get(9)?;

                    Ok(ArtifactRecord {
                        id: ArtifactId::from(id_str),
                        task_id: task_id_str.map(TaskId::from),
                        workspace_id: ws_id_str.map(WorkspaceId::from),
                        name,
                        path,
                        artifact_type,
                        size,
                        checksum,
                        created_at,
                        verification_status,
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

    fn list_artifacts_by_workspace(
        &self,
        workspace_id: &WorkspaceId,
    ) -> Result<Vec<ArtifactRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, task_id, workspace_id, name, path, artifact_type, size, checksum, created_at, verification_status FROM artifacts WHERE workspace_id = ?1 ORDER BY created_at DESC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map(params![workspace_id.as_str()], |row| {
                    let id_str: String = row.get(0)?;
                    let task_id_str: Option<String> = row.get(1)?;
                    let ws_id_str: Option<String> = row.get(2)?;
                    let name: String = row.get(3)?;
                    let path: String = row.get(4)?;
                    let artifact_type: String = row.get(5)?;
                    let size: u64 = row.get(6)?;
                    let checksum: Option<String> = row.get(7)?;
                    let created_at: u64 = row.get(8)?;
                    let verification_status: String = row.get(9)?;

                    Ok(ArtifactRecord {
                        id: ArtifactId::from(id_str),
                        task_id: task_id_str.map(TaskId::from),
                        workspace_id: ws_id_str.map(WorkspaceId::from),
                        name,
                        path,
                        artifact_type,
                        size,
                        checksum,
                        created_at,
                        verification_status,
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

    fn update_verification_status(
        &self,
        id: &ArtifactId,
        status: &str,
    ) -> Result<bool, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute(
                    "UPDATE artifacts SET verification_status = ?1 WHERE id = ?2",
                    params![status, id.as_str()],
                )
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;
            Ok(count > 0)
        })
    }

    fn delete_artifact(&self, id: &ArtifactId) -> Result<bool, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute("DELETE FROM artifacts WHERE id = ?1", params![id.as_str()])
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;
            Ok(count > 0)
        })
    }
}
