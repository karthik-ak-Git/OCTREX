use crate::db::manager::DatabaseManager;
use crate::db::models::ProviderRecord;
use crate::error::OctrexError;
use rusqlite::params;

pub trait ProviderRepository: Send + Sync {
    fn create_provider(&self, provider: &ProviderRecord) -> Result<ProviderRecord, OctrexError>;
    fn get_provider(&self, id: &str) -> Result<Option<ProviderRecord>, OctrexError>;
    fn list_providers(&self) -> Result<Vec<ProviderRecord>, OctrexError>;
    fn update_provider(&self, provider: &ProviderRecord) -> Result<ProviderRecord, OctrexError>;
    fn delete_provider(&self, id: &str) -> Result<bool, OctrexError>;
}

pub struct SqliteProviderRepository {
    db: DatabaseManager,
}

impl SqliteProviderRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
}

impl ProviderRepository for SqliteProviderRepository {
    fn create_provider(&self, provider: &ProviderRecord) -> Result<ProviderRecord, OctrexError> {
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO providers (id, name, provider_type, enabled, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    provider.id,
                    provider.name,
                    provider.provider_type,
                    if provider.enabled { 1 } else { 0 },
                    provider.created_at,
                    provider.updated_at
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to insert provider: {}", e),
            })?;
            Ok(provider.clone())
        })
    }

    fn get_provider(&self, id: &str) -> Result<Option<ProviderRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, name, provider_type, enabled, created_at, updated_at FROM providers WHERE id = ?1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let result = stmt.query_row(params![id], |row| {
                let id_str: String = row.get(0)?;
                let name: String = row.get(1)?;
                let provider_type: String = row.get(2)?;
                let enabled_val: i32 = row.get(3)?;
                let created_at: u64 = row.get(4)?;
                let updated_at: u64 = row.get(5)?;

                Ok(ProviderRecord {
                    id: id_str,
                    name,
                    provider_type,
                    enabled: enabled_val != 0,
                    created_at,
                    updated_at,
                })
            });

            match result {
                Ok(p) => Ok(Some(p)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(OctrexError::Internal { message: e.to_string() }),
            }
        })
    }

    fn list_providers(&self) -> Result<Vec<ProviderRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, name, provider_type, enabled, created_at, updated_at FROM providers ORDER BY name ASC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map([], |row| {
                    let id_str: String = row.get(0)?;
                    let name: String = row.get(1)?;
                    let provider_type: String = row.get(2)?;
                    let enabled_val: i32 = row.get(3)?;
                    let created_at: u64 = row.get(4)?;
                    let updated_at: u64 = row.get(5)?;

                    Ok(ProviderRecord {
                        id: id_str,
                        name,
                        provider_type,
                        enabled: enabled_val != 0,
                        created_at,
                        updated_at,
                    })
                })
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let mut providers = Vec::new();
            for r in rows {
                providers.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
            }
            Ok(providers)
        })
    }

    fn update_provider(&self, provider: &ProviderRecord) -> Result<ProviderRecord, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute(
                    "UPDATE providers SET name = ?1, provider_type = ?2, enabled = ?3, updated_at = ?4 WHERE id = ?5",
                    params![
                        provider.name,
                        provider.provider_type,
                        if provider.enabled { 1 } else { 0 },
                        provider.updated_at,
                        provider.id
                    ],
                )
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            if count == 0 {
                Err(OctrexError::NotFound {
                    resource: format!("Provider with id '{}'", provider.id),
                })
            } else {
                Ok(provider.clone())
            }
        })
    }

    fn delete_provider(&self, id: &str) -> Result<bool, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute("DELETE FROM providers WHERE id = ?1", params![id])
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;
            Ok(count > 0)
        })
    }
}
