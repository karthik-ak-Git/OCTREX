use super::errors::OrchestrationError;
use super::types::{OrchestratorState, TaskPlan};
use crate::db::manager::DatabaseManager;
use crate::error::OctrexError;
use crate::ids::TaskId;

pub struct TaskRecoveryManager;

impl TaskRecoveryManager {
    /// Save task checkpoint state to persistent database
    pub fn save_checkpoint(
        db: &DatabaseManager,
        plan: &TaskPlan,
        state: OrchestratorState,
    ) -> Result<(), OrchestrationError> {
        let plan_json = serde_json::to_string(plan).map_err(|e| OrchestrationError::Internal {
            message: format!("Failed to serialize task plan: {}", e),
        })?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO task_plans (id, task_id, objective, constraints_json, steps_json, current_step, status, version, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT(id) DO UPDATE SET
                   steps_json = excluded.steps_json,
                   current_step = excluded.current_step,
                   status = excluded.status,
                   version = excluded.version,
                   updated_at = excluded.updated_at",
                rusqlite::params![
                    plan.id,
                    plan.task_id.as_str(),
                    plan.objective,
                    serde_json::to_string(&plan.constraints).unwrap_or_default(),
                    plan_json,
                    plan.current_step as u32,
                    state.to_string(),
                    plan.version,
                    plan.created_at,
                    now
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Database checkpoint error: {}", e),
            })?;
            Ok(())
        })
        .map_err(Into::into)
    }

    /// Load persisted task plan from SQLite database
    pub fn load_plan(
        db: &DatabaseManager,
        task_id: &TaskId,
    ) -> Result<Option<TaskPlan>, OrchestrationError> {
        let json_opt: Option<String> = db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT steps_json FROM task_plans WHERE task_id = ?1 ORDER BY version DESC LIMIT 1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let res = stmt.query_row(rusqlite::params![task_id.as_str()], |row| {
                let json_str: String = row.get(0)?;
                Ok(json_str)
            });

            match res {
                Ok(json_str) => Ok(Some(json_str)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(OctrexError::Internal { message: e.to_string() }),
            }
        }).map_err(|e| OrchestrationError::Internal { message: e.to_string() })?;

        if let Some(json_str) = json_opt {
            let plan: TaskPlan =
                serde_json::from_str(&json_str).map_err(|e| OrchestrationError::Internal {
                    message: format!("Failed to deserialize plan checkpoint: {}", e),
                })?;
            Ok(Some(plan))
        } else {
            Ok(None)
        }
    }
}
