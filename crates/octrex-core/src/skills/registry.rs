use crate::db::manager::DatabaseManager;
use crate::skills::errors::SkillError;
use crate::skills::types::{now_millis, SkillDefinition, SkillStatus};
use crate::skills::validator;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory registry backed by the shared `skills` table (Phase 2) plus a
/// `skill_versions` table (Phase 14). No duplicate skill database is created.
#[derive(Clone)]
pub struct SkillRegistry {
    db: Arc<DatabaseManager>,
    cache: Arc<RwLock<HashMap<String, SkillDefinition>>>,
    metrics: Arc<RwLock<HashMap<String, crate::skills::types::SkillMetrics>>>,
}

impl SkillRegistry {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self {
            db,
            cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register(&self, mut def: SkillDefinition) -> Result<SkillDefinition, SkillError> {
        crate::skills::security::SkillSecurity::scan_for_injection(&def)?;
        crate::skills::provenance::check_boundary_bypass(
            &def.capabilities_required,
            &def.allowed_tools,
        )?;
        // Validation determines eligibility; invalid => Disabled.
        let valid = validator::validate_skill(&def).is_ok();
        def.status = validator::post_validation_status(&def, valid);
        def.updated_at = now_millis();
        if def.created_at == 0 {
            def.created_at = def.updated_at;
        }

        // Persist index row into the pre-existing `skills` table (reuse, no duplicate).
        let trust_label = format!("rank:{}", def.source.trust_rank());
        self.db
            .with_conn(|conn| {
                conn.execute(
                    "INSERT INTO skills (id, name, version, description, source, enabled, trust_level, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                     ON CONFLICT(id) DO UPDATE SET name=excluded.name, version=excluded.version, description=excluded.description, source=excluded.source, enabled=excluded.enabled, trust_level=excluded.trust_level, updated_at=excluded.updated_at",
                    rusqlite::params![
                        def.id,
                        def.name,
                        def.version,
                        def.description,
                        def.source.to_string(),
                        if def.status == SkillStatus::Active { 1 } else { 0 },
                        trust_label,
                        def.created_at as i64,
                        def.updated_at as i64,
                    ],
                )
                .map_err(|e| crate::error::OctrexError::Internal {
                    message: format!("Failed to persist skill index: {}", e),
                })?;
                // Full declarative definition goes to skill_versions (no silent repair).
                let full = serde_json::to_string(&def).map_err(|e| {
                    crate::error::OctrexError::Internal {
                        message: e.to_string(),
                    }
                })?;
                let validation = if valid { "VALID" } else { "INVALID" };
                conn.execute(
                    "INSERT INTO skill_versions (skill_id, version, definition_json, status, validation, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                     ON CONFLICT(skill_id, version) DO UPDATE SET definition_json=excluded.definition_json, status=excluded.status, validation=excluded.validation",
                    rusqlite::params![
                        def.id,
                        def.version,
                        full,
                        def.status.to_string(),
                        validation,
                        def.updated_at as i64,
                    ],
                )
                .map_err(|e| crate::error::OctrexError::Internal {
                    message: format!("Failed to persist skill version: {}", e),
                })?;
                Ok(())
            })
            .map_err(|e| SkillError::Persistence {
                message: e.to_string(),
            })?;

        {
            let mut guard = self.cache.write().unwrap();
            guard.insert(def.id.clone(), def.clone());
        }
        Ok(def)
    }

    pub fn get(&self, id: &str) -> Option<SkillDefinition> {
        {
            let guard = self.cache.read().unwrap();
            if let Some(d) = guard.get(id) {
                return Some(d.clone());
            }
        }
        // Fallback to latest persisted version.
        let loaded: Option<SkillDefinition> = self
            .db
            .with_conn(|conn| {
                let mut stmt = conn
                    .prepare(
                        "SELECT definition_json FROM skill_versions WHERE skill_id = ?1 ORDER BY created_at DESC LIMIT 1",
                    )
                    .map_err(|e| crate::error::OctrexError::Internal {
                        message: e.to_string(),
                    })?;
                let res: Result<String, rusqlite::Error> =
                    stmt.query_row(rusqlite::params![id], |row| row.get(0));
                match res {
                    Ok(json) => Ok(Some(json)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(crate::error::OctrexError::Internal {
                        message: e.to_string(),
                    }),
                }
            })
            .ok()
            .flatten()
            .and_then(|json| serde_json::from_str(&json).ok());
        if let Some(def) = loaded.clone() {
            let mut guard = self.cache.write().unwrap();
            guard.insert(id.to_string(), def);
        }
        loaded
    }

    pub fn get_version(&self, id: &str, version: &str) -> Option<SkillDefinition> {
        if let Some(def) = self.get(id) {
            if def.version == version {
                return Some(def);
            }
        }
        self.db
            .with_conn(|conn| {
                let mut stmt = conn
                    .prepare(
                        "SELECT definition_json FROM skill_versions WHERE skill_id = ?1 AND version = ?2 LIMIT 1",
                    )
                    .map_err(|e| crate::error::OctrexError::Internal {
                        message: e.to_string(),
                    })?;
                let res: Result<String, rusqlite::Error> =
                    stmt.query_row(rusqlite::params![id, version], |row| row.get(0));
                match res {
                    Ok(json) => Ok(Some(json)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(crate::error::OctrexError::Internal {
                        message: e.to_string(),
                    }),
                }
            })
            .ok()
            .flatten()
            .and_then(|json| serde_json::from_str(&json).ok())
    }

    pub fn list(&self) -> Vec<SkillDefinition> {
        let guard = self.cache.read().unwrap();
        let mut out: Vec<SkillDefinition> = guard.values().cloned().collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    pub fn set_status(&self, id: &str, status: SkillStatus) -> Result<SkillDefinition, SkillError> {
        let mut def = self.get(id).ok_or_else(|| SkillError::NotFound {
            skill_id: id.to_string(),
        })?;
        // Untrusted sources can only become Active through explicit approval.
        if status == SkillStatus::Active
            && matches!(
                def.source,
                crate::skills::types::SkillSource::Imported
                    | crate::skills::types::SkillSource::ModelGenerated
                    | crate::skills::types::SkillSource::External
            )
            && def.provenance.approval_status != "APPROVED"
        {
            return Err(SkillError::SecurityViolation {
                reason: "Untrusted skill requires APPROVED provenance before activation"
                    .to_string(),
            });
        }
        def.status = status;
        def.updated_at = now_millis();
        {
            let mut guard = self.cache.write().unwrap();
            guard.insert(id.to_string(), def.clone());
        }
        let full = serde_json::to_string(&def).unwrap_or_default();
        let db = self.db.clone();
        let id_owned = id.to_string();
        let version = def.version.clone();
        let status_str = def.status.to_string();
        let _ = db.with_conn(|conn| {
            conn.execute(
                "UPDATE skills SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![
                    if status == SkillStatus::Active { 1 } else { 0 },
                    def.updated_at as i64,
                    id_owned,
                ],
            )
            .map_err(|e| crate::error::OctrexError::Internal {
                message: e.to_string(),
            })?;
            conn.execute(
                "UPDATE skill_versions SET definition_json = ?1, status = ?2 WHERE skill_id = ?3 AND version = ?4",
                rusqlite::params![full, status_str, id_owned, version],
            )
            .map_err(|e| crate::error::OctrexError::Internal {
                message: e.to_string(),
            })?;
            Ok(())
        });
        Ok(def)
    }

    pub fn approve(&self, id: &str, approver: &str) -> Result<SkillDefinition, SkillError> {
        let mut def = self.get(id).ok_or_else(|| SkillError::NotFound {
            skill_id: id.to_string(),
        })?;
        def.provenance.approval_status = "APPROVED".to_string();
        def.provenance.created_by =
            format!("{} (approved by {})", def.provenance.created_by, approver);
        def.provenance.last_updated = now_millis();
        // Re-validate after approval.
        match validator::validate_skill(&def) {
            Ok(()) => {
                def.status = SkillStatus::Active;
                def.provenance.validation_status = "VALID".to_string();
            }
            Err(e) => {
                def.status = SkillStatus::Disabled;
                def.provenance.validation_status = format!("INVALID: {}", e);
            }
        }
        def.updated_at = now_millis();
        {
            let mut guard = self.cache.write().unwrap();
            guard.insert(id.to_string(), def.clone());
        }
        Ok(def)
    }

    pub fn record_use(&self, id: &str, task_id: &str, success: bool, duration_ms: u64) {
        {
            let mut guard = self.cache.write().unwrap();
            if let Some(def) = guard.get_mut(id) {
                if !def.provenance.tasks_used_in.contains(&task_id.to_string()) {
                    def.provenance.tasks_used_in.push(task_id.to_string());
                }
            }
        }
        {
            let mut m = self.metrics.write().unwrap();
            let entry = m.entry(id.to_string()).or_default();
            entry.total_runs += 1;
            if success {
                entry.successful_runs += 1;
            }
            entry.total_duration_ms += duration_ms;
        }
    }

    pub fn metrics(&self, id: &str) -> crate::skills::types::SkillMetrics {
        let guard = self.metrics.read().unwrap();
        guard.get(id).cloned().unwrap_or_default()
    }
}
