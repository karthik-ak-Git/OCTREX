use crate::db::manager::DatabaseManager;
use crate::workflows::errors::WorkflowError;
use crate::workflows::types::{now_millis, WorkflowDefinition, WorkflowRun, WorkflowRunStatus};
use crate::workflows::validator;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct WorkflowRegistry {
    db: Arc<DatabaseManager>,
    cache: Arc<RwLock<HashMap<String, WorkflowDefinition>>>,
}

impl WorkflowRegistry {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self {
            db,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register(
        &self,
        mut def: WorkflowDefinition,
    ) -> Result<WorkflowDefinition, WorkflowError> {
        let valid = validator::validate_workflow(&def).is_ok();
        def.status = validator::post_validation_status(&def, valid);
        def.updated_at = now_millis();
        if def.created_at == 0 {
            def.created_at = def.updated_at;
        }
        let full = serde_json::to_string(&def).map_err(|e| WorkflowError::Persistence {
            message: e.to_string(),
        })?;
        let validation = if valid { "VALID" } else { "INVALID" };
        let status_str = def.status.to_string();
        self.db
            .with_conn(|conn| {
                conn.execute(
                    "INSERT INTO workflow_definitions (id, name, version, description, definition_json, status, source, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                     ON CONFLICT(id, version) DO UPDATE SET name=excluded.name, description=excluded.description, definition_json=excluded.definition_json, status=excluded.status, source=excluded.source, updated_at=excluded.updated_at",
                    rusqlite::params![
                        def.id,
                        def.name,
                        def.version,
                        def.description,
                        full,
                        status_str,
                        def.source.to_string(),
                        def.created_at as i64,
                        def.updated_at as i64,
                    ],
                )
                .map_err(|e| crate::error::OctrexError::Internal {
                    message: format!("Failed to persist workflow: {}", e),
                })?;
                Ok(())
            })
            .map_err(|e| WorkflowError::Persistence {
                message: e.to_string(),
            })?;
        // Mark unused var to satisfy clippy in case of future use.
        let _ = validation;
        {
            let mut guard = self.cache.write().unwrap();
            guard.insert(def.id.clone(), def.clone());
        }
        Ok(def)
    }

    pub fn get(&self, id: &str) -> Option<WorkflowDefinition> {
        {
            let guard = self.cache.read().unwrap();
            if let Some(d) = guard.get(id) {
                return Some(d.clone());
            }
        }
        let loaded: Option<WorkflowDefinition> = self
            .db
            .with_conn(|conn| {
                let mut stmt = conn
                    .prepare("SELECT definition_json FROM workflow_definitions WHERE id = ?1 ORDER BY updated_at DESC LIMIT 1")
                    .map_err(|e| crate::error::OctrexError::Internal { message: e.to_string() })?;
                let res: Result<String, rusqlite::Error> =
                    stmt.query_row(rusqlite::params![id], |row| row.get(0));
                match res {
                    Ok(j) => Ok(Some(j)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(crate::error::OctrexError::Internal { message: e.to_string() }),
                }
            })
            .ok()
            .flatten()
            .and_then(|j| serde_json::from_str(&j).ok());
        if let Some(def) = loaded.clone() {
            let mut guard = self.cache.write().unwrap();
            guard.insert(id.to_string(), def);
        }
        loaded
    }

    pub fn get_version(&self, id: &str, version: &str) -> Option<WorkflowDefinition> {
        self.db
            .with_conn(|conn| {
                let mut stmt = conn
                    .prepare("SELECT definition_json FROM workflow_definitions WHERE id = ?1 AND version = ?2 LIMIT 1")
                    .map_err(|e| crate::error::OctrexError::Internal { message: e.to_string() })?;
                let res: Result<String, rusqlite::Error> =
                    stmt.query_row(rusqlite::params![id, version], |row| row.get(0));
                match res {
                    Ok(j) => Ok(Some(j)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(crate::error::OctrexError::Internal { message: e.to_string() }),
                }
            })
            .ok()
            .flatten()
            .and_then(|j| serde_json::from_str(&j).ok())
    }

    pub fn list(&self) -> Vec<WorkflowDefinition> {
        let guard = self.cache.read().unwrap();
        let mut out: Vec<WorkflowDefinition> = guard.values().cloned().collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    pub fn record_run(&self, run: &WorkflowRun) -> Result<(), WorkflowError> {
        let outputs = run
            .outputs
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_default();
        self.db
            .with_conn(|conn| {
                conn.execute(
                    "INSERT INTO workflow_runs (id, workflow_id, workflow_version, task_id, session_id, workspace_id, status, inputs_json, outputs_json, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                     ON CONFLICT(id) DO UPDATE SET status=excluded.status, outputs_json=excluded.outputs_json, updated_at=excluded.updated_at",
                    rusqlite::params![
                        run.id,
                        run.workflow_id,
                        run.workflow_version,
                        run.task_id,
                        run.session_id,
                        run.workspace_id,
                        run.status.to_string(),
                        run.inputs.to_string(),
                        outputs,
                        run.created_at as i64,
                        run.updated_at as i64,
                    ],
                )
                .map_err(|e| crate::error::OctrexError::Internal { message: e.to_string() })?;
                Ok(())
            })
            .map_err(|e| WorkflowError::Persistence { message: e.to_string() })?;
        Ok(())
    }

    pub fn get_run(&self, id: &str) -> Option<WorkflowRun> {
        self.db
            .with_conn(|conn| {
                let mut stmt = conn
                    .prepare("SELECT id, workflow_id, workflow_version, task_id, session_id, workspace_id, status, inputs_json, outputs_json, created_at, updated_at FROM workflow_runs WHERE id = ?1")
                    .map_err(|e| crate::error::OctrexError::Internal { message: e.to_string() })?;
                let res = stmt.query_row(rusqlite::params![id], |row| {
                    let status_str: String = row.get(6)?;
                    let inputs_str: String = row.get(7)?;
                    let outputs_str: String = row.get(8)?;
                    let status = match status_str.as_str() {
                        "RUNNING" => WorkflowRunStatus::Running,
                        "COMPLETED" => WorkflowRunStatus::Completed,
                        "FAILED" => WorkflowRunStatus::Failed,
                        "CANCELLED" => WorkflowRunStatus::Cancelled,
                        _ => WorkflowRunStatus::Pending,
                    };
                    Ok(WorkflowRun {
                        id: row.get(0)?,
                        workflow_id: row.get(1)?,
                        workflow_version: row.get(2)?,
                        task_id: row.get(3)?,
                        session_id: row.get(4)?,
                        workspace_id: row.get(5)?,
                        status,
                        inputs: serde_json::from_str(&inputs_str).unwrap_or(serde_json::Value::Null),
                        outputs: if outputs_str.is_empty() { None } else { serde_json::from_str(&outputs_str).ok() },
                        created_at: row.get::<_, i64>(9)? as u64,
                        updated_at: row.get::<_, i64>(10)? as u64,
                    })
                });
                match res {
                    Ok(r) => Ok(Some(r)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(crate::error::OctrexError::Internal { message: e.to_string() }),
                }
            })
            .ok()
            .flatten()
    }
}
