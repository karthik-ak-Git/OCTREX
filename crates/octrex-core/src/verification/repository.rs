use crate::db::DatabaseManager;
use crate::error::OctrexError;
use crate::verification::errors::VerificationError;
use crate::verification::types::{VerificationCheck, VerificationResult, VerificationStatus};
use rusqlite::params;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct VerificationRunRecord {
    pub verification_id: String,
    pub task_id: String,
    pub step_id: Option<String>,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    pub status: VerificationStatus,
    pub confidence: f64,
    pub repair_attempt: u32,
    pub warnings_json: String,
    pub failures_json: String,
    pub created_at: u64,
}

pub struct SqliteVerificationRepository {
    db: DatabaseManager,
}

impl SqliteVerificationRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }

    pub fn save_run(
        &self,
        result: &VerificationResult,
        workspace_id: Option<&str>,
        session_id: Option<&str>,
    ) -> Result<(), VerificationError> {
        self.db
            .with_conn(|conn| {
                conn.execute(
                    "INSERT INTO verification_runs (id, task_id, step_id, workspace_id, session_id, status, confidence, repair_attempt, warnings_json, failures_json, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        result.verification_id,
                        result.task_id,
                        result.step_id,
                        workspace_id,
                        session_id,
                        result.status.to_string(),
                        result.confidence,
                        result.repair_attempt as i64,
                        serde_json::to_string(&result.warnings).unwrap_or_else(|_| "[]".to_string()),
                        serde_json::to_string(&result.failures).unwrap_or_else(|_| "[]".to_string()),
                        result.created_at,
                    ],
                )
                .map_err(|e| OctrexError::Internal {
                    message: format!("Failed to insert verification run: {}", e),
                })?;

                for check in &result.checks {
                    conn.execute(
                        "INSERT INTO verification_checks (id, verification_id, name, check_type, status, severity, expected, actual, evidence_ref, message)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                        params![
                            check.id,
                            result.verification_id,
                            check.name,
                            check.check_type.to_string(),
                            check.status.to_string(),
                            check.severity.to_string(),
                            check.expected,
                            check.actual,
                            check.evidence_ref,
                            check.message,
                        ],
                    )
                    .map_err(|e| OctrexError::Internal {
                        message: format!("Failed to insert verification check: {}", e),
                    })?;
                }
                Ok(())
            })
            .map_err(|e| VerificationError::Persistence {
                message: e.to_string(),
            })?;
        Ok(())
    }

    pub fn get_run(
        &self,
        verification_id: &str,
    ) -> Result<Option<VerificationRunRecord>, VerificationError> {
        self.db
            .with_conn(|conn| {
                let mut stmt = conn
                    .prepare("SELECT id, task_id, step_id, workspace_id, session_id, status, confidence, repair_attempt, warnings_json, failures_json, created_at FROM verification_runs WHERE id = ?1")
                    .map_err(|e| OctrexError::Internal { message: e.to_string() })?;
                let res = stmt.query_row(params![verification_id], |row| {
                    Ok(VerificationRunRecord {
                        verification_id: row.get(0)?,
                        task_id: row.get(1)?,
                        step_id: row.get(2)?,
                        workspace_id: row.get(3)?,
                        session_id: row.get(4)?,
                        status: {
                            let s: String = row.get(5)?;
                            VerificationStatus::from_str(&s).unwrap_or(VerificationStatus::Unknown)
                        },
                        confidence: row.get(6)?,
                        repair_attempt: {
                            let v: i64 = row.get(7)?;
                            v.max(0) as u32
                        },
                        warnings_json: row.get(8)?,
                        failures_json: row.get(9)?,
                        created_at: row.get(10)?,
                    })
                });
                match res {
                    Ok(r) => Ok(Some(r)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(OctrexError::Internal { message: e.to_string() }),
                }
            })
            .map_err(|e| VerificationError::Persistence {
                message: e.to_string(),
            })
    }

    pub fn get_checks(
        &self,
        verification_id: &str,
    ) -> Result<Vec<VerificationCheck>, VerificationError> {
        self.db
            .with_conn(|conn| {
                let mut stmt = conn
                    .prepare("SELECT id, name, check_type, status, severity, expected, actual, evidence_ref, message FROM verification_checks WHERE verification_id = ?1 ORDER BY rowid ASC")
                    .map_err(|e| OctrexError::Internal { message: e.to_string() })?;
                let rows = stmt
                    .query_map(params![verification_id], |row| {
                        let id: String = row.get(0)?;
                        let name: String = row.get(1)?;
                        let check_type_s: String = row.get(2)?;
                        let status_s: String = row.get(3)?;
                        let severity_s: String = row.get(4)?;
                        let expected: String = row.get(5)?;
                        let actual: String = row.get(6)?;
                        let evidence_ref: Option<String> = row.get(7)?;
                        let message: String = row.get(8)?;
                        Ok((id, name, check_type_s, status_s, severity_s, expected, actual, evidence_ref, message))
                    })
                    .map_err(|e| OctrexError::Internal { message: e.to_string() })?;
                let mut out = Vec::new();
                for r in rows {
                    let (id, name, ct_s, st_s, sev_s, expected, actual, evidence_ref, message) =
                        r.map_err(|e| OctrexError::Internal { message: e.to_string() })?;
                    let check_type = match ct_s.as_str() {
                        "FILE_EXISTS" => crate::verification::types::VerificationCheckType::FileExists,
                        "FILE_CONTENT" => crate::verification::types::VerificationCheckType::FileContent,
                        "FILE_MODIFIED" => crate::verification::types::VerificationCheckType::FileModified,
                        "FILE_CREATED" => crate::verification::types::VerificationCheckType::FileCreated,
                        "ARTIFACT_EXISTS" => crate::verification::types::VerificationCheckType::ArtifactExists,
                        "ARTIFACT_VALID" => crate::verification::types::VerificationCheckType::ArtifactValid,
                        "COMMAND_SUCCEEDED" => crate::verification::types::VerificationCheckType::CommandSucceeded,
                        "TEST_PASSED" => crate::verification::types::VerificationCheckType::TestPassed,
                        "BUILD_PASSED" => crate::verification::types::VerificationCheckType::BuildPassed,
                        "TOOL_SUCCEEDED" => crate::verification::types::VerificationCheckType::ToolSucceeded,
                        "OUTPUT_SCHEMA" => crate::verification::types::VerificationCheckType::OutputSchema,
                        "REQUIRED_FIELD" => crate::verification::types::VerificationCheckType::RequiredField,
                        "CONSTRAINT_SATISFIED" => crate::verification::types::VerificationCheckType::ConstraintSatisfied,
                        "STEP_COMPLETED" => crate::verification::types::VerificationCheckType::StepCompleted,
                        "DEPENDENCY_SATISFIED" => crate::verification::types::VerificationCheckType::DependencySatisfied,
                        "USER_REQUIREMENT" => crate::verification::types::VerificationCheckType::UserRequirement,
                        _ => crate::verification::types::VerificationCheckType::NoBlockingError,
                    };
                    let status = match st_s.as_str() {
                        "PASSED" => crate::verification::types::CheckStatus::Passed,
                        "WARNING" => crate::verification::types::CheckStatus::Warning,
                        "FAILED" => crate::verification::types::CheckStatus::Failed,
                        "BLOCKED" => crate::verification::types::CheckStatus::Blocked,
                        "SKIPPED" => crate::verification::types::CheckStatus::Skipped,
                        _ => crate::verification::types::CheckStatus::Unknown,
                    };
                    let severity = match sev_s.as_str() {
                        "WARNING" => crate::verification::types::CheckSeverity::Warning,
                        "CRITICAL" => crate::verification::types::CheckSeverity::Critical,
                        "BLOCKING" => crate::verification::types::CheckSeverity::Blocking,
                        _ => crate::verification::types::CheckSeverity::Info,
                    };
                    out.push(VerificationCheck { id, name, check_type, status, severity, expected, actual, evidence_ref, message });
                }
                Ok(out)
            })
            .map_err(|e| VerificationError::Persistence {
                message: e.to_string(),
            })
    }

    pub fn list_runs_by_task(
        &self,
        task_id: &str,
    ) -> Result<Vec<VerificationRunRecord>, VerificationError> {
        self.db
            .with_conn(|conn| {
                let mut stmt = conn
                    .prepare("SELECT id, task_id, step_id, workspace_id, session_id, status, confidence, repair_attempt, warnings_json, failures_json, created_at FROM verification_runs WHERE task_id = ?1 ORDER BY created_at DESC")
                    .map_err(|e| OctrexError::Internal { message: e.to_string() })?;
                let rows = stmt
                    .query_map(params![task_id], |row| {
                        Ok(VerificationRunRecord {
                            verification_id: row.get(0)?,
                            task_id: row.get(1)?,
                            step_id: row.get(2)?,
                            workspace_id: row.get(3)?,
                            session_id: row.get(4)?,
                            status: {
                                let s: String = row.get(5)?;
                                VerificationStatus::from_str(&s).unwrap_or(VerificationStatus::Unknown)
                            },
                            confidence: row.get(6)?,
                            repair_attempt: {
                                let v: i64 = row.get(7)?;
                                v.max(0) as u32
                            },
                            warnings_json: row.get(8)?,
                            failures_json: row.get(9)?,
                            created_at: row.get(10)?,
                        })
                    })
                    .map_err(|e| OctrexError::Internal { message: e.to_string() })?;
                let mut out = Vec::new();
                for r in rows {
                    out.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
                }
                Ok(out)
            })
            .map_err(|e| VerificationError::Persistence {
                message: e.to_string(),
            })
    }

    pub fn count_runs_by_task(&self, task_id: &str) -> Result<u32, VerificationError> {
        self.db
            .with_conn(|conn| {
                let count: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM verification_runs WHERE task_id = ?1",
                        params![task_id],
                        |row| row.get(0),
                    )
                    .map_err(|e| OctrexError::Internal {
                        message: e.to_string(),
                    })?;
                Ok(count.max(0) as u32)
            })
            .map_err(|e| VerificationError::Persistence {
                message: e.to_string(),
            })
    }
}
