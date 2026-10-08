use crate::db::manager::DatabaseManager;
use crate::error::{AppErrorResponse, OctrexError};
use crate::ids::{RequestId, SessionId, TaskId, WorkspaceId};
use crate::task::{Task, TaskStatus};
use rusqlite::params;

pub trait TaskRepository: Send + Sync {
    fn create_task(&self, task: &Task) -> Result<Task, OctrexError>;
    fn get_task(&self, id: &TaskId) -> Result<Option<Task>, OctrexError>;
    fn list_tasks(&self) -> Result<Vec<Task>, OctrexError>;
    fn list_active_tasks(&self) -> Result<Vec<Task>, OctrexError>;
    fn update_task(&self, task: &Task) -> Result<Task, OctrexError>;
    fn delete_task(&self, id: &TaskId) -> Result<bool, OctrexError>;
}

pub struct SqliteTaskRepository {
    db: DatabaseManager,
}

impl SqliteTaskRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
}

impl TaskRepository for SqliteTaskRepository {
    fn create_task(&self, task: &Task) -> Result<Task, OctrexError> {
        self.db.with_conn(|conn| {
            let req_id_str = task.request_id.as_ref().map(|r| r.as_str());
            let sess_id_str = task.session_id.as_ref().map(|s| s.as_str());
            let ws_id_str = task.workspace_id.as_ref().map(|w| w.as_str());

            let status_str = match task.status {
                TaskStatus::Created => "CREATED",
                TaskStatus::Running => "RUNNING",
                TaskStatus::Paused => "PAUSED",
                TaskStatus::Cancelled => "CANCELLED",
                TaskStatus::Completed => "COMPLETED",
                TaskStatus::Failed => "FAILED",
            };

            let err_code = task.error.as_ref().map(|e| e.code.as_str());
            let err_msg = task.error.as_ref().map(|e| e.message.as_str());

            conn.execute(
                "INSERT INTO tasks (id, request_id, session_id, workspace_id, title, status, created_at, updated_at, error_code, error_message)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    task.id.as_str(),
                    req_id_str,
                    sess_id_str,
                    ws_id_str,
                    task.title,
                    status_str,
                    task.created_at,
                    task.updated_at,
                    err_code,
                    err_msg
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to insert task: {}", e),
            })?;
            Ok(task.clone())
        })
    }

    fn get_task(&self, id: &TaskId) -> Result<Option<Task>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, request_id, session_id, workspace_id, title, status, created_at, updated_at, error_code, error_message
                     FROM tasks WHERE id = ?1",
                )
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let result = stmt.query_row(params![id.as_str()], |row| {
                let id_str: String = row.get(0)?;
                let req_id_str: Option<String> = row.get(1)?;
                let sess_id_str: Option<String> = row.get(2)?;
                let ws_id_str: Option<String> = row.get(3)?;
                let title: String = row.get(4)?;
                let status_str: String = row.get(5)?;
                let created_at: u64 = row.get(6)?;
                let updated_at: u64 = row.get(7)?;
                let err_code: Option<String> = row.get(8)?;
                let err_msg: Option<String> = row.get(9)?;

                let status = match status_str.to_uppercase().as_str() {
                    "RUNNING" => TaskStatus::Running,
                    "PAUSED" => TaskStatus::Paused,
                    "CANCELLED" => TaskStatus::Cancelled,
                    "COMPLETED" => TaskStatus::Completed,
                    "FAILED" => TaskStatus::Failed,
                    _ => TaskStatus::Created,
                };

                let error = match (err_code, err_msg) {
                    (Some(code_str), Some(msg)) => {
                        let code = match code_str.as_str() {
                            "VALIDATION_ERROR" => crate::error::ErrorCode::ValidationError,
                            "NOT_FOUND" => crate::error::ErrorCode::NotFound,
                            "CONFLICT" => crate::error::ErrorCode::Conflict,
                            "PERMISSION_DENIED" => crate::error::ErrorCode::PermissionDenied,
                            "POLICY_DENIED" => crate::error::ErrorCode::PolicyDenied,
                            "PRIVACY_BLOCKED" => crate::error::ErrorCode::PrivacyBlocked,
                            "NETWORK_BLOCKED" => crate::error::ErrorCode::NetworkBlocked,
                            "MODEL_UNAVAILABLE" => crate::error::ErrorCode::ModelUnavailable,
                            "HARDWARE_INCOMPATIBLE" => crate::error::ErrorCode::HardwareIncompatible,
                            "CONTEXT_OVERFLOW" => crate::error::ErrorCode::ContextOverflow,
                            "TOOL_FAILED" => crate::error::ErrorCode::ToolFailed,
                            "VERIFICATION_FAILED" => crate::error::ErrorCode::VerificationFailed,
                            "FILESYSTEM_ERROR" => crate::error::ErrorCode::FilesystemError,
                            "PROCESS_ERROR" => crate::error::ErrorCode::ProcessError,
                            "MCP_ERROR" => crate::error::ErrorCode::McpError,
                            "PROVIDER_ERROR" => crate::error::ErrorCode::ProviderError,
                            "TIMEOUT" => crate::error::ErrorCode::Timeout,
                            "CANCELLED" => crate::error::ErrorCode::Cancelled,
                            _ => crate::error::ErrorCode::InternalError,
                        };
                        Some(AppErrorResponse {
                            code,
                            message: msg,
                            details: None,
                            correlation_id: req_id_str.as_ref().map(|r| RequestId::from(r.as_str())),
                        })
                    }
                    _ => None,
                };

                Ok(Task {
                    id: TaskId::from(id_str),
                    request_id: req_id_str.map(RequestId::from),
                    session_id: sess_id_str.map(SessionId::from),
                    workspace_id: ws_id_str.map(WorkspaceId::from),
                    title,
                    status,
                    created_at,
                    updated_at,
                    error,
                })
            });

            match result {
                Ok(t) => Ok(Some(t)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(OctrexError::Internal { message: e.to_string() }),
            }
        })
    }

    fn list_tasks(&self) -> Result<Vec<Task>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, request_id, session_id, workspace_id, title, status, created_at, updated_at, error_code, error_message
                     FROM tasks ORDER BY updated_at DESC",
                )
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map([], |row| {
                    let id_str: String = row.get(0)?;
                    let req_id_str: Option<String> = row.get(1)?;
                    let sess_id_str: Option<String> = row.get(2)?;
                    let ws_id_str: Option<String> = row.get(3)?;
                    let title: String = row.get(4)?;
                    let status_str: String = row.get(5)?;
                    let created_at: u64 = row.get(6)?;
                    let updated_at: u64 = row.get(7)?;
                    let err_code: Option<String> = row.get(8)?;
                    let err_msg: Option<String> = row.get(9)?;

                    let status = match status_str.to_uppercase().as_str() {
                        "RUNNING" => TaskStatus::Running,
                        "PAUSED" => TaskStatus::Paused,
                        "CANCELLED" => TaskStatus::Cancelled,
                        "COMPLETED" => TaskStatus::Completed,
                        "FAILED" => TaskStatus::Failed,
                        _ => TaskStatus::Created,
                    };

                    let error = match (err_code, err_msg) {
                        (Some(code_str), Some(msg)) => {
                            let code = match code_str.as_str() {
                                "VALIDATION_ERROR" => crate::error::ErrorCode::ValidationError,
                                "NOT_FOUND" => crate::error::ErrorCode::NotFound,
                                _ => crate::error::ErrorCode::InternalError,
                            };
                            Some(AppErrorResponse {
                                code,
                                message: msg,
                                details: None,
                                correlation_id: req_id_str.as_ref().map(|r| RequestId::from(r.as_str())),
                            })
                        }
                        _ => None,
                    };

                    Ok(Task {
                        id: TaskId::from(id_str),
                        request_id: req_id_str.map(RequestId::from),
                        session_id: sess_id_str.map(SessionId::from),
                        workspace_id: ws_id_str.map(WorkspaceId::from),
                        title,
                        status,
                        created_at,
                        updated_at,
                        error,
                    })
                })
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let mut tasks = Vec::new();
            for r in rows {
                tasks.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
            }
            Ok(tasks)
        })
    }

    fn list_active_tasks(&self) -> Result<Vec<Task>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, request_id, session_id, workspace_id, title, status, created_at, updated_at, error_code, error_message
                     FROM tasks WHERE status IN ('CREATED', 'RUNNING', 'PAUSED') ORDER BY updated_at DESC",
                )
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map([], |row| {
                    let id_str: String = row.get(0)?;
                    let req_id_str: Option<String> = row.get(1)?;
                    let sess_id_str: Option<String> = row.get(2)?;
                    let ws_id_str: Option<String> = row.get(3)?;
                    let title: String = row.get(4)?;
                    let status_str: String = row.get(5)?;
                    let created_at: u64 = row.get(6)?;
                    let updated_at: u64 = row.get(7)?;

                    let status = match status_str.to_uppercase().as_str() {
                        "RUNNING" => TaskStatus::Running,
                        "PAUSED" => TaskStatus::Paused,
                        _ => TaskStatus::Created,
                    };

                    Ok(Task {
                        id: TaskId::from(id_str),
                        request_id: req_id_str.map(RequestId::from),
                        session_id: sess_id_str.map(SessionId::from),
                        workspace_id: ws_id_str.map(WorkspaceId::from),
                        title,
                        status,
                        created_at,
                        updated_at,
                        error: None,
                    })
                })
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let mut tasks = Vec::new();
            for r in rows {
                tasks.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
            }
            Ok(tasks)
        })
    }

    fn update_task(&self, task: &Task) -> Result<Task, OctrexError> {
        self.db.with_conn(|conn| {
            let status_str = match task.status {
                TaskStatus::Created => "CREATED",
                TaskStatus::Running => "RUNNING",
                TaskStatus::Paused => "PAUSED",
                TaskStatus::Cancelled => "CANCELLED",
                TaskStatus::Completed => "COMPLETED",
                TaskStatus::Failed => "FAILED",
            };

            let err_code = task.error.as_ref().map(|e| e.code.as_str());
            let err_msg = task.error.as_ref().map(|e| e.message.as_str());

            let count = conn
                .execute(
                    "UPDATE tasks SET status = ?1, updated_at = ?2, error_code = ?3, error_message = ?4 WHERE id = ?5",
                    params![
                        status_str,
                        task.updated_at,
                        err_code,
                        err_msg,
                        task.id.as_str()
                    ],
                )
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            if count == 0 {
                Err(OctrexError::NotFound {
                    resource: format!("Task with id '{}'", task.id),
                })
            } else {
                Ok(task.clone())
            }
        })
    }

    fn delete_task(&self, id: &TaskId) -> Result<bool, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute("DELETE FROM tasks WHERE id = ?1", params![id.as_str()])
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;
            Ok(count > 0)
        })
    }
}
