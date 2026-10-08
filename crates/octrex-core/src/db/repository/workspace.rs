use crate::db::manager::DatabaseManager;
use crate::error::OctrexError;
use crate::ids::WorkspaceId;
use crate::workspace::Workspace;
use rusqlite::params;
use std::path::PathBuf;

pub trait WorkspaceRepository: Send + Sync {
    fn create_workspace(&self, ws: &Workspace) -> Result<Workspace, OctrexError>;
    fn get_workspace(&self, id: &WorkspaceId) -> Result<Option<Workspace>, OctrexError>;
    fn list_workspaces(&self) -> Result<Vec<Workspace>, OctrexError>;
    fn update_workspace(&self, ws: &Workspace) -> Result<Workspace, OctrexError>;
    fn delete_workspace(&self, id: &WorkspaceId) -> Result<bool, OctrexError>;
}

pub struct SqliteWorkspaceRepository {
    db: DatabaseManager,
}

impl SqliteWorkspaceRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
}

impl WorkspaceRepository for SqliteWorkspaceRepository {
    fn create_workspace(&self, ws: &Workspace) -> Result<Workspace, OctrexError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO workspaces (id, name, path, classification, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    ws.id.as_str(),
                    ws.name,
                    ws.path.to_string_lossy().to_string(),
                    ws.classification,
                    ws.created_at,
                    ws.updated_at
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to create workspace in database: {}", e),
            })?;
            Ok(ws.clone())
        })
    }

    fn get_workspace(&self, id: &WorkspaceId) -> Result<Option<Workspace>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, path, COALESCE(classification, 'PUBLIC'), created_at, updated_at FROM workspaces WHERE id = ?1",
                )
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;

            let result = stmt.query_row(params![id.as_str()], |row| {
                let id_str: String = row.get(0)?;
                let name: String = row.get(1)?;
                let path_str: String = row.get(2)?;
                let classification: String = row.get(3)?;
                let created_at: u64 = row.get(4)?;
                let updated_at: u64 = row.get(5)?;

                Ok(Workspace {
                    id: WorkspaceId::from(id_str),
                    name,
                    path: PathBuf::from(path_str),
                    classification,
                    created_at,
                    updated_at,
                })
            });

            match result {
                Ok(ws) => Ok(Some(ws)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(OctrexError::Internal {
                    message: e.to_string(),
                }),
            }
        })
    }

    fn list_workspaces(&self) -> Result<Vec<Workspace>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, name, path, COALESCE(classification, 'PUBLIC'), created_at, updated_at FROM workspaces ORDER BY updated_at DESC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map([], |row| {
                    let id_str: String = row.get(0)?;
                    let name: String = row.get(1)?;
                    let path_str: String = row.get(2)?;
                    let classification: String = row.get(3)?;
                    let created_at: u64 = row.get(4)?;
                    let updated_at: u64 = row.get(5)?;

                    Ok(Workspace {
                        id: WorkspaceId::from(id_str),
                        name,
                        path: PathBuf::from(path_str),
                        classification,
                        created_at,
                        updated_at,
                    })
                })
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let mut workspaces = Vec::new();
            for r in rows {
                workspaces.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
            }
            Ok(workspaces)
        })
    }

    fn update_workspace(&self, ws: &Workspace) -> Result<Workspace, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute(
                    "UPDATE workspaces SET name = ?1, path = ?2, classification = ?3, updated_at = ?4 WHERE id = ?5",
                    params![
                        ws.name,
                        ws.path.to_string_lossy().to_string(),
                        ws.classification,
                        ws.updated_at,
                        ws.id.as_str()
                    ],
                )
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;

            if count == 0 {
                Err(OctrexError::NotFound {
                    resource: format!("Workspace with id '{}'", ws.id),
                })
            } else {
                Ok(ws.clone())
            }
        })
    }

    fn delete_workspace(&self, id: &WorkspaceId) -> Result<bool, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute("DELETE FROM workspaces WHERE id = ?1", params![id.as_str()])
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;
            Ok(count > 0)
        })
    }
}
