use crate::db::manager::DatabaseManager;
use crate::db::models::{TaskStep, TaskStepStatus};
use crate::error::OctrexError;
use crate::ids::TaskId;
use rusqlite::params;
use std::str::FromStr;

pub trait TaskStepRepository: Send + Sync {
    fn create_step(&self, step: &TaskStep) -> Result<TaskStep, OctrexError>;
    fn get_step(&self, id: &str) -> Result<Option<TaskStep>, OctrexError>;
    fn list_steps_by_task(&self, task_id: &TaskId) -> Result<Vec<TaskStep>, OctrexError>;
    fn update_step(&self, step: &TaskStep) -> Result<TaskStep, OctrexError>;
    fn delete_step(&self, id: &str) -> Result<bool, OctrexError>;
}

pub struct SqliteTaskStepRepository {
    db: DatabaseManager,
}

impl SqliteTaskStepRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
}

impl TaskStepRepository for SqliteTaskStepRepository {
    fn create_step(&self, step: &TaskStep) -> Result<TaskStep, OctrexError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO task_steps (id, task_id, sequence, objective, status, created_at, started_at, completed_at, error)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    step.id,
                    step.task_id.as_str(),
                    step.sequence,
                    step.objective,
                    step.status.to_string(),
                    step.created_at,
                    step.started_at,
                    step.completed_at,
                    step.error
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to insert task step: {}", e),
            })?;
            Ok(step.clone())
        })
    }

    fn get_step(&self, id: &str) -> Result<Option<TaskStep>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, task_id, sequence, objective, status, created_at, started_at, completed_at, error FROM task_steps WHERE id = ?1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let result = stmt.query_row(params![id], |row| {
                let id_str: String = row.get(0)?;
                let task_id_str: String = row.get(1)?;
                let sequence: u32 = row.get(2)?;
                let objective: String = row.get(3)?;
                let status_str: String = row.get(4)?;
                let created_at: u64 = row.get(5)?;
                let started_at: Option<u64> = row.get(6)?;
                let completed_at: Option<u64> = row.get(7)?;
                let error: Option<String> = row.get(8)?;

                let status = TaskStepStatus::from_str(&status_str)
                    .unwrap_or(TaskStepStatus::Pending);

                Ok(TaskStep {
                    id: id_str,
                    task_id: TaskId::from(task_id_str),
                    sequence,
                    objective,
                    status,
                    created_at,
                    started_at,
                    completed_at,
                    error,
                })
            });

            match result {
                Ok(s) => Ok(Some(s)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(OctrexError::Internal { message: e.to_string() }),
            }
        })
    }

    fn list_steps_by_task(&self, task_id: &TaskId) -> Result<Vec<TaskStep>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, task_id, sequence, objective, status, created_at, started_at, completed_at, error FROM task_steps WHERE task_id = ?1 ORDER BY sequence ASC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map(params![task_id.as_str()], |row| {
                    let id_str: String = row.get(0)?;
                    let task_id_str: String = row.get(1)?;
                    let sequence: u32 = row.get(2)?;
                    let objective: String = row.get(3)?;
                    let status_str: String = row.get(4)?;
                    let created_at: u64 = row.get(5)?;
                    let started_at: Option<u64> = row.get(6)?;
                    let completed_at: Option<u64> = row.get(7)?;
                    let error: Option<String> = row.get(8)?;

                    let status = TaskStepStatus::from_str(&status_str)
                        .unwrap_or(TaskStepStatus::Pending);

                    Ok(TaskStep {
                        id: id_str,
                        task_id: TaskId::from(task_id_str),
                        sequence,
                        objective,
                        status,
                        created_at,
                        started_at,
                        completed_at,
                        error,
                    })
                })
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let mut steps = Vec::new();
            for r in rows {
                steps.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
            }
            Ok(steps)
        })
    }

    fn update_step(&self, step: &TaskStep) -> Result<TaskStep, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute(
                    "UPDATE task_steps SET sequence = ?1, objective = ?2, status = ?3, started_at = ?4, completed_at = ?5, error = ?6 WHERE id = ?7",
                    params![
                        step.sequence,
                        step.objective,
                        step.status.to_string(),
                        step.started_at,
                        step.completed_at,
                        step.error,
                        step.id
                    ],
                )
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            if count == 0 {
                Err(OctrexError::NotFound {
                    resource: format!("TaskStep with id '{}'", step.id),
                })
            } else {
                Ok(step.clone())
            }
        })
    }

    fn delete_step(&self, id: &str) -> Result<bool, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute("DELETE FROM task_steps WHERE id = ?1", params![id])
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;
            Ok(count > 0)
        })
    }
}
