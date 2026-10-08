use crate::db::manager::DatabaseManager;
use crate::db::models::SkillRecord;
use crate::error::OctrexError;
use rusqlite::params;

pub trait SkillRepository: Send + Sync {
    fn create_skill(&self, skill: &SkillRecord) -> Result<SkillRecord, OctrexError>;
    fn get_skill(&self, id: &str) -> Result<Option<SkillRecord>, OctrexError>;
    fn list_skills(&self) -> Result<Vec<SkillRecord>, OctrexError>;
    fn update_skill(&self, skill: &SkillRecord) -> Result<SkillRecord, OctrexError>;
    fn delete_skill(&self, id: &str) -> Result<bool, OctrexError>;
}

pub struct SqliteSkillRepository {
    db: DatabaseManager,
}

impl SqliteSkillRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
}

impl SkillRepository for SqliteSkillRepository {
    fn create_skill(&self, skill: &SkillRecord) -> Result<SkillRecord, OctrexError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO skills (id, name, version, description, source, enabled, trust_level, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    skill.id,
                    skill.name,
                    skill.version,
                    skill.description,
                    skill.source,
                    if skill.enabled { 1 } else { 0 },
                    skill.trust_level,
                    skill.created_at,
                    skill.updated_at
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to insert skill: {}", e),
            })?;
            Ok(skill.clone())
        })
    }

    fn get_skill(&self, id: &str) -> Result<Option<SkillRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, name, version, description, source, enabled, trust_level, created_at, updated_at FROM skills WHERE id = ?1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let result = stmt.query_row(params![id], |row| {
                let id_str: String = row.get(0)?;
                let name: String = row.get(1)?;
                let version: String = row.get(2)?;
                let description: String = row.get(3)?;
                let source: String = row.get(4)?;
                let enabled_val: i32 = row.get(5)?;
                let trust_level: String = row.get(6)?;
                let created_at: u64 = row.get(7)?;
                let updated_at: u64 = row.get(8)?;

                Ok(SkillRecord {
                    id: id_str,
                    name,
                    version,
                    description,
                    source,
                    enabled: enabled_val != 0,
                    trust_level,
                    created_at,
                    updated_at,
                })
            });

            match result {
                Ok(s) => Ok(Some(s)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(OctrexError::Internal { message: e.to_string() }),
            }
        })
    }

    fn list_skills(&self) -> Result<Vec<SkillRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, name, version, description, source, enabled, trust_level, created_at, updated_at FROM skills ORDER BY name ASC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map([], |row| {
                    let id_str: String = row.get(0)?;
                    let name: String = row.get(1)?;
                    let version: String = row.get(2)?;
                    let description: String = row.get(3)?;
                    let source: String = row.get(4)?;
                    let enabled_val: i32 = row.get(5)?;
                    let trust_level: String = row.get(6)?;
                    let created_at: u64 = row.get(7)?;
                    let updated_at: u64 = row.get(8)?;

                    Ok(SkillRecord {
                        id: id_str,
                        name,
                        version,
                        description,
                        source,
                        enabled: enabled_val != 0,
                        trust_level,
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

    fn update_skill(&self, skill: &SkillRecord) -> Result<SkillRecord, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute(
                    "UPDATE skills SET name = ?1, version = ?2, description = ?3, source = ?4, enabled = ?5, trust_level = ?6, updated_at = ?7 WHERE id = ?8",
                    params![
                        skill.name,
                        skill.version,
                        skill.description,
                        skill.source,
                        if skill.enabled { 1 } else { 0 },
                        skill.trust_level,
                        skill.updated_at,
                        skill.id
                    ],
                )
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            if count == 0 {
                Err(OctrexError::NotFound {
                    resource: format!("Skill with id '{}'", skill.id),
                })
            } else {
                Ok(skill.clone())
            }
        })
    }

    fn delete_skill(&self, id: &str) -> Result<bool, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute("DELETE FROM skills WHERE id = ?1", params![id])
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;
            Ok(count > 0)
        })
    }
}
