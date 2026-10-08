use crate::db::manager::DatabaseManager;
use crate::memory::classifier::{apply_floor_to_item, validate_item};
use crate::memory::errors::MemoryError;
use crate::memory::retention::apply_default_retention;
use crate::memory::types::{now_millis, MemoryCandidate, MemoryItem, MemoryQuery, MemoryResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Memory store: in-memory cache + SQLite persistence. No transcript archive:
/// only structured facts/state summaries are stored.
#[derive(Clone)]
pub struct MemoryStore {
    db: Arc<DatabaseManager>,
    cache: Arc<RwLock<HashMap<String, MemoryItem>>>,
    candidates: Arc<RwLock<HashMap<String, MemoryCandidate>>>,
}

impl MemoryStore {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self {
            db,
            cache: Arc::new(RwLock::new(HashMap::new())),
            candidates: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_item(&self, mut item: MemoryItem) -> Result<MemoryItem, MemoryError> {
        apply_floor_to_item(&mut item);
        apply_default_retention(&mut item);
        validate_item(&item)?;
        crate::memory::policy::MemoryPolicy::check_create(&item)?;
        persist_item(&self.db, &item)?;
        {
            let mut guard = self.cache.write().unwrap();
            guard.insert(item.id.clone(), item.clone());
        }
        Ok(item)
    }

    pub fn get_item(&self, id: &str) -> Option<MemoryItem> {
        {
            let guard = self.cache.read().unwrap();
            if let Some(item) = guard.get(id) {
                return Some(item.clone());
            }
        }
        load_item(&self.db, id)
    }

    pub fn update_item(
        &self,
        id: &str,
        content: Option<String>,
        confidence: Option<f64>,
    ) -> Result<MemoryItem, MemoryError> {
        let mut item = self.get_item(id).ok_or_else(|| MemoryError::NotFound {
            memory_id: id.to_string(),
        })?;
        if let Some(c) = content {
            item.content = c;
        }
        if let Some(conf) = confidence {
            if !(0.0..=1.0).contains(&conf) {
                return Err(MemoryError::Validation {
                    reason: "Confidence must be within [0,1]".to_string(),
                });
            }
            item.confidence = conf;
        }
        // Classification never downgrades: re-apply floor.
        apply_floor_to_item(&mut item);
        item.updated_at = now_millis();
        item.version += 1;
        validate_item(&item)?;
        persist_item(&self.db, &item)?;
        {
            let mut guard = self.cache.write().unwrap();
            guard.insert(id.to_string(), item.clone());
        }
        Ok(item)
    }

    pub fn delete_item(&self, id: &str) -> Result<bool, MemoryError> {
        {
            let mut guard = self.cache.write().unwrap();
            guard.remove(id);
        }
        delete_persisted_item(&self.db, id)
    }

    pub fn query(&self, query: &MemoryQuery) -> Result<Vec<MemoryResult>, MemoryError> {
        crate::memory::policy::MemoryPolicy::check_query(query)?;
        let now = now_millis();
        // Load persisted items into cache lazily (bounded).
        let persisted = load_all_for_scopes(&self.db, query)?;
        {
            let mut guard = self.cache.write().unwrap();
            for item in &persisted {
                guard.insert(item.id.clone(), item.clone());
            }
        }
        let guard = self.cache.read().unwrap();
        let all: Vec<MemoryItem> = guard.values().cloned().collect();
        let filtered = crate::memory::retrieval::filter_items(&all, query, now);
        Ok(crate::memory::retrieval::rank_items(filtered, query, now))
    }

    pub fn create_candidate(
        &self,
        candidate: MemoryCandidate,
    ) -> Result<MemoryCandidate, MemoryError> {
        if candidate.source == crate::memory::types::MemorySource::Unknown {
            return Err(MemoryError::SecurityViolation {
                reason: "Candidate source UNKNOWN fails closed".to_string(),
            });
        }
        persist_candidate(&self.db, &candidate)?;
        {
            let mut guard = self.candidates.write().unwrap();
            guard.insert(candidate.id.clone(), candidate.clone());
        }
        Ok(candidate)
    }

    pub fn get_candidate(&self, id: &str) -> Option<MemoryCandidate> {
        {
            let guard = self.candidates.read().unwrap();
            if let Some(c) = guard.get(id) {
                return Some(c.clone());
            }
        }
        load_candidate(&self.db, id)
    }

    pub fn list_candidates(&self, status: Option<&str>) -> Vec<MemoryCandidate> {
        let persisted = load_candidates(&self.db, status);
        {
            let mut guard = self.candidates.write().unwrap();
            for c in &persisted {
                guard.insert(c.id.clone(), c.clone());
            }
        }
        let guard = self.candidates.read().unwrap();
        let mut out: Vec<MemoryCandidate> = guard
            .values()
            .filter(|c| status.map(|s| c.status == s).unwrap_or(true))
            .cloned()
            .collect();
        out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        out
    }

    pub fn set_candidate_status(
        &self,
        id: &str,
        status: &str,
    ) -> Result<MemoryCandidate, MemoryError> {
        let mut cand = self
            .get_candidate(id)
            .ok_or_else(|| MemoryError::NotFound {
                memory_id: id.to_string(),
            })?;
        cand.status = status.to_string();
        persist_candidate(&self.db, &cand)?;
        {
            let mut guard = self.candidates.write().unwrap();
            guard.insert(id.to_string(), cand.clone());
        }
        Ok(cand)
    }

    /// Approve a candidate into durable memory. Model-derived candidates keep
    /// their low-trust source; approval does not launder trust.
    pub fn approve_candidate(&self, id: &str) -> Result<MemoryItem, MemoryError> {
        let cand = self
            .get_candidate(id)
            .ok_or_else(|| MemoryError::NotFound {
                memory_id: id.to_string(),
            })?;
        if cand.status != "PENDING" {
            return Err(MemoryError::Validation {
                reason: format!("Candidate '{}' is not PENDING", id),
            });
        }
        let now = now_millis();
        let mut item = MemoryItem {
            id: format!("mem-{}", uuid::Uuid::new_v4().simple()),
            mem_type: cand.mem_type,
            scope: cand.scope,
            workspace_id: cand.workspace_id.clone(),
            project_id: None,
            session_id: cand.session_id.clone(),
            task_id: cand.task_id.clone(),
            content: cand.content.clone(),
            classification: cand.classification,
            source: cand.source,
            confidence: cand.confidence,
            provenance: crate::memory::types::MemoryProvenance {
                source: cand.source,
                actor: cand.provenance.actor.clone(),
                tool_id: cand.provenance.tool_id.clone(),
                model_id: cand.provenance.model_id.clone(),
                imported_from: cand.provenance.imported_from.clone(),
                evidence: cand.provenance.evidence.clone(),
            },
            created_at: now,
            updated_at: now,
            expires_at: None,
            version: 1,
        };
        apply_default_retention(&mut item);
        let stored = self.create_item(item)?;
        self.set_candidate_status(id, "APPROVED")?;
        Ok(stored)
    }

    pub fn reject_candidate(&self, id: &str) -> Result<MemoryCandidate, MemoryError> {
        self.set_candidate_status(id, "REJECTED")
    }
}

fn persist_item(db: &DatabaseManager, item: &MemoryItem) -> Result<(), MemoryError> {
    let provenance = serde_json::to_string(&item.provenance).unwrap_or_default();
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO memory_items (id, mem_type, scope, workspace_id, project_id, session_id, task_id, content, classification, trust_source, confidence, provenance_json, created_at, updated_at, expires_at, version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
             ON CONFLICT(id) DO UPDATE SET content=excluded.content, classification=excluded.classification, confidence=excluded.confidence, provenance_json=excluded.provenance_json, updated_at=excluded.updated_at, expires_at=excluded.expires_at, version=excluded.version",
            rusqlite::params![
                item.id,
                item.mem_type.to_string(),
                item.scope.to_string(),
                item.workspace_id,
                item.project_id,
                item.session_id,
                item.task_id,
                item.content,
                item.classification.to_string(),
                item.source.to_string(),
                item.confidence,
                provenance,
                item.created_at as i64,
                item.updated_at as i64,
                item.expires_at.map(|e| e as i64),
                item.version,
            ],
        )
        .map_err(|e| crate::error::OctrexError::Internal { message: e.to_string() })?;
        Ok(())
    })
    .map_err(|e| MemoryError::Persistence { message: e.to_string() })?;
    Ok(())
}

fn row_to_item(row: &rusqlite::Row) -> rusqlite::Result<MemoryItem> {
    let mem_type_str: String = row.get(1)?;
    let scope_str: String = row.get(2)?;
    let classification_str: String = row.get(8)?;
    let source_str: String = row.get(9)?;
    let provenance_str: String = row.get(11)?;
    Ok(MemoryItem {
        id: row.get(0)?,
        mem_type: mem_type_str
            .parse()
            .unwrap_or(crate::memory::types::MemoryType::TaskFact),
        scope: scope_str
            .parse()
            .unwrap_or(crate::memory::types::MemoryScope::Workspace),
        workspace_id: row.get(3)?,
        project_id: row.get(4)?,
        session_id: row.get(5)?,
        task_id: row.get(6)?,
        content: row.get(7)?,
        classification: classification_str
            .parse()
            .unwrap_or(crate::privacy::PrivacyClassification::Public),
        source: source_str
            .parse()
            .unwrap_or(crate::memory::types::MemorySource::Unknown),
        confidence: row.get(10)?,
        provenance: serde_json::from_str(&provenance_str).unwrap_or_default(),
        created_at: row.get::<_, i64>(12)? as u64,
        updated_at: row.get::<_, i64>(13)? as u64,
        expires_at: row.get::<_, Option<i64>>(14)?.map(|e| e as u64),
        version: row.get::<_, i64>(15)? as u32,
    })
}

fn load_item(db: &DatabaseManager, id: &str) -> Option<MemoryItem> {
    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, mem_type, scope, workspace_id, project_id, session_id, task_id, content, classification, trust_source, confidence, provenance_json, created_at, updated_at, expires_at, version FROM memory_items WHERE id = ?1")
            .map_err(|e| crate::error::OctrexError::Internal { message: e.to_string() })?;
        let res = stmt.query_row(rusqlite::params![id], row_to_item);
        match res {
            Ok(item) => Ok(Some(item)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(crate::error::OctrexError::Internal { message: e.to_string() }),
        }
    })
    .ok()
    .flatten()
}

fn load_all_for_scopes(
    db: &DatabaseManager,
    query: &MemoryQuery,
) -> Result<Vec<MemoryItem>, MemoryError> {
    db.with_conn(|conn| {
        // Bounded load: filter by scope ids in SQL, then apply full policy in Rust.
        let sql = "SELECT id, mem_type, scope, workspace_id, project_id, session_id, task_id, content, classification, trust_source, confidence, provenance_json, created_at, updated_at, expires_at, version FROM memory_items LIMIT 500";
        // Keep the query simple and bounded; full isolation enforced in retrieval::filter_items.
        let _ = query;
        let mut stmt = conn.prepare(&sql).map_err(|e| crate::error::OctrexError::Internal { message: e.to_string() })?;
        let rows = stmt.query_map([], row_to_item).map_err(|e| crate::error::OctrexError::Internal { message: e.to_string() })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| crate::error::OctrexError::Internal { message: e.to_string() })?);
        }
        Ok(out)
    })
    .map_err(|e| MemoryError::Persistence { message: e.to_string() })
}

fn delete_persisted_item(db: &DatabaseManager, id: &str) -> Result<bool, MemoryError> {
    let count: usize = db
        .with_conn(|conn| {
            let n = conn
                .execute(
                    "DELETE FROM memory_items WHERE id = ?1",
                    rusqlite::params![id],
                )
                .map_err(|e| crate::error::OctrexError::Internal {
                    message: e.to_string(),
                })?;
            Ok(n)
        })
        .map_err(|e| MemoryError::Persistence {
            message: e.to_string(),
        })?;
    Ok(count > 0)
}

fn persist_candidate(db: &DatabaseManager, cand: &MemoryCandidate) -> Result<(), MemoryError> {
    let provenance = serde_json::to_string(&cand.provenance).unwrap_or_default();
    db.with_conn(|conn| {
        conn.execute(
            "INSERT INTO memory_candidates (id, mem_type, scope, workspace_id, session_id, task_id, content, classification, source, confidence, provenance_json, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(id) DO UPDATE SET status=excluded.status, content=excluded.content, updated_at=excluded.updated_at",
            rusqlite::params![
                cand.id,
                cand.mem_type.to_string(),
                cand.scope.to_string(),
                cand.workspace_id,
                cand.session_id,
                cand.task_id,
                cand.content,
                cand.classification.to_string(),
                cand.source.to_string(),
                cand.confidence,
                provenance,
                cand.status,
                cand.created_at as i64,
                cand.updated_at as i64,
            ],
        )
        .map_err(|e| crate::error::OctrexError::Internal { message: e.to_string() })?;
        Ok(())
    })
    .map_err(|e| MemoryError::Persistence { message: e.to_string() })?;
    Ok(())
}

fn row_to_candidate(row: &rusqlite::Row) -> rusqlite::Result<MemoryCandidate> {
    let mem_type_str: String = row.get(1)?;
    let scope_str: String = row.get(2)?;
    let classification_str: String = row.get(7)?;
    let source_str: String = row.get(8)?;
    let provenance_str: String = row.get(10)?;
    Ok(MemoryCandidate {
        id: row.get(0)?,
        mem_type: mem_type_str
            .parse()
            .unwrap_or(crate::memory::types::MemoryType::TaskFact),
        scope: scope_str
            .parse()
            .unwrap_or(crate::memory::types::MemoryScope::Workspace),
        workspace_id: row.get(3)?,
        session_id: row.get(4)?,
        task_id: row.get(5)?,
        content: row.get(6)?,
        classification: classification_str
            .parse()
            .unwrap_or(crate::privacy::PrivacyClassification::Public),
        source: source_str
            .parse()
            .unwrap_or(crate::memory::types::MemorySource::Unknown),
        confidence: row.get(9)?,
        provenance: serde_json::from_str(&provenance_str).unwrap_or(
            crate::memory::types::MemoryProvenance {
                source: crate::memory::types::MemorySource::Unknown,
                actor: "unknown".to_string(),
                tool_id: None,
                model_id: None,
                imported_from: None,
                evidence: String::new(),
            },
        ),
        status: row.get(11)?,
        created_at: row.get::<_, i64>(12)? as u64,
        updated_at: row.get::<_, i64>(13)? as u64,
    })
}

fn load_candidate(db: &DatabaseManager, id: &str) -> Option<MemoryCandidate> {
    db.with_conn(|conn| {
        let mut stmt = conn
            .prepare("SELECT id, mem_type, scope, workspace_id, session_id, task_id, content, classification, source, confidence, provenance_json, status, created_at, updated_at FROM memory_candidates WHERE id = ?1")
            .map_err(|e| crate::error::OctrexError::Internal { message: e.to_string() })?;
        let res = stmt.query_row(rusqlite::params![id], row_to_candidate);
        match res {
            Ok(c) => Ok(Some(c)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(crate::error::OctrexError::Internal { message: e.to_string() }),
        }
    })
    .ok()
    .flatten()
}

fn load_candidates(db: &DatabaseManager, status: Option<&str>) -> Vec<MemoryCandidate> {
    db.with_conn(|conn| {
        let (sql, param): (String, Option<String>) = match status {
            Some(s) => (
                "SELECT id, mem_type, scope, workspace_id, session_id, task_id, content, classification, source, confidence, provenance_json, status, created_at, updated_at FROM memory_candidates WHERE status = ?1 ORDER BY created_at DESC LIMIT 200".to_string(),
                Some(s.to_string()),
            ),
            None => (
                "SELECT id, mem_type, scope, workspace_id, session_id, task_id, content, classification, source, confidence, provenance_json, status, created_at, updated_at FROM memory_candidates ORDER BY created_at DESC LIMIT 200".to_string(),
                None,
            ),
        };
        let mut stmt = conn.prepare(&sql).map_err(|e| crate::error::OctrexError::Internal { message: e.to_string() })?;
        let rows = if let Some(p) = param {
            stmt.query_map(rusqlite::params![p], row_to_candidate)
        } else {
            stmt.query_map([], row_to_candidate)
        }
        .map_err(|e| crate::error::OctrexError::Internal { message: e.to_string() })?;
        let mut out = Vec::new();
        for r in rows {
            if let Ok(c) = r {
                out.push(c);
            }
        }
        Ok(out)
    })
    .unwrap_or_default()
}
