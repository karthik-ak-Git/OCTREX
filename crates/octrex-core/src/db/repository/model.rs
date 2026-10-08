use crate::db::manager::DatabaseManager;
use crate::db::models::ModelRecord;
use crate::error::OctrexError;
use rusqlite::params;

pub trait ModelRepository: Send + Sync {
    fn create_model(&self, model: &ModelRecord) -> Result<ModelRecord, OctrexError>;
    fn get_model(&self, id: &str) -> Result<Option<ModelRecord>, OctrexError>;
    fn list_models_by_provider(&self, provider_id: &str) -> Result<Vec<ModelRecord>, OctrexError>;
    fn list_all_models(&self) -> Result<Vec<ModelRecord>, OctrexError>;
    fn update_model(&self, model: &ModelRecord) -> Result<ModelRecord, OctrexError>;
    fn delete_model(&self, id: &str) -> Result<bool, OctrexError>;
}

pub struct SqliteModelRepository {
    db: DatabaseManager,
}

impl SqliteModelRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
}

impl ModelRepository for SqliteModelRepository {
    fn create_model(&self, model: &ModelRecord) -> Result<ModelRecord, OctrexError> {
        self.db.with_conn(|conn| {
            let caps_str = model.capabilities.as_ref().map(|c| serde_json::to_string(c).unwrap_or_default());
            let meta_str = model.metadata.as_ref().map(|m| serde_json::to_string(m).unwrap_or_default());

            conn.execute(
                "INSERT INTO models (id, provider_id, model_identifier, display_name, enabled, local_or_remote, context_window, capabilities, metadata, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    model.id,
                    model.provider_id,
                    model.model_identifier,
                    model.display_name,
                    if model.enabled { 1 } else { 0 },
                    model.local_or_remote,
                    model.context_window,
                    caps_str,
                    meta_str,
                    model.created_at,
                    model.updated_at
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to insert model: {}", e),
            })?;
            Ok(model.clone())
        })
    }

    fn get_model(&self, id: &str) -> Result<Option<ModelRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, provider_id, model_identifier, display_name, enabled, local_or_remote, context_window, capabilities, metadata, created_at, updated_at FROM models WHERE id = ?1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let result = stmt.query_row(params![id], |row| {
                let id_str: String = row.get(0)?;
                let provider_id: String = row.get(1)?;
                let model_identifier: String = row.get(2)?;
                let display_name: String = row.get(3)?;
                let enabled_val: i32 = row.get(4)?;
                let local_or_remote: String = row.get(5)?;
                let context_window: u32 = row.get(6)?;
                let caps_str: Option<String> = row.get(7)?;
                let meta_str: Option<String> = row.get(8)?;
                let created_at: u64 = row.get(9)?;
                let updated_at: u64 = row.get(10)?;

                let capabilities = caps_str.and_then(|s| serde_json::from_str(&s).ok());
                let metadata = meta_str.and_then(|s| serde_json::from_str(&s).ok());

                Ok(ModelRecord {
                    id: id_str,
                    provider_id,
                    model_identifier,
                    display_name,
                    enabled: enabled_val != 0,
                    local_or_remote,
                    context_window,
                    capabilities,
                    metadata,
                    created_at,
                    updated_at,
                })
            });

            match result {
                Ok(m) => Ok(Some(m)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(OctrexError::Internal { message: e.to_string() }),
            }
        })
    }

    fn list_models_by_provider(&self, provider_id: &str) -> Result<Vec<ModelRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, provider_id, model_identifier, display_name, enabled, local_or_remote, context_window, capabilities, metadata, created_at, updated_at FROM models WHERE provider_id = ?1 ORDER BY display_name ASC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map(params![provider_id], |row| {
                    let id_str: String = row.get(0)?;
                    let provider_id: String = row.get(1)?;
                    let model_identifier: String = row.get(2)?;
                    let display_name: String = row.get(3)?;
                    let enabled_val: i32 = row.get(4)?;
                    let local_or_remote: String = row.get(5)?;
                    let context_window: u32 = row.get(6)?;
                    let caps_str: Option<String> = row.get(7)?;
                    let meta_str: Option<String> = row.get(8)?;
                    let created_at: u64 = row.get(9)?;
                    let updated_at: u64 = row.get(10)?;

                    let capabilities = caps_str.and_then(|s| serde_json::from_str(&s).ok());
                    let metadata = meta_str.and_then(|s| serde_json::from_str(&s).ok());

                    Ok(ModelRecord {
                        id: id_str,
                        provider_id,
                        model_identifier,
                        display_name,
                        enabled: enabled_val != 0,
                        local_or_remote,
                        context_window,
                        capabilities,
                        metadata,
                        created_at,
                        updated_at,
                    })
                })
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let mut models = Vec::new();
            for r in rows {
                models.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
            }
            Ok(models)
        })
    }

    fn list_all_models(&self) -> Result<Vec<ModelRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, provider_id, model_identifier, display_name, enabled, local_or_remote, context_window, capabilities, metadata, created_at, updated_at FROM models ORDER BY display_name ASC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map([], |row| {
                    let id_str: String = row.get(0)?;
                    let provider_id: String = row.get(1)?;
                    let model_identifier: String = row.get(2)?;
                    let display_name: String = row.get(3)?;
                    let enabled_val: i32 = row.get(4)?;
                    let local_or_remote: String = row.get(5)?;
                    let context_window: u32 = row.get(6)?;
                    let caps_str: Option<String> = row.get(7)?;
                    let meta_str: Option<String> = row.get(8)?;
                    let created_at: u64 = row.get(9)?;
                    let updated_at: u64 = row.get(10)?;

                    let capabilities = caps_str.and_then(|s| serde_json::from_str(&s).ok());
                    let metadata = meta_str.and_then(|s| serde_json::from_str(&s).ok());

                    Ok(ModelRecord {
                        id: id_str,
                        provider_id,
                        model_identifier,
                        display_name,
                        enabled: enabled_val != 0,
                        local_or_remote,
                        context_window,
                        capabilities,
                        metadata,
                        created_at,
                        updated_at,
                    })
                })
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let mut models = Vec::new();
            for r in rows {
                models.push(r.map_err(|e| OctrexError::Internal { message: e.to_string() })?);
            }
            Ok(models)
        })
    }

    fn update_model(&self, model: &ModelRecord) -> Result<ModelRecord, OctrexError> {
        self.db.with_conn(|conn| {
            let caps_str = model.capabilities.as_ref().map(|c| serde_json::to_string(c).unwrap_or_default());
            let meta_str = model.metadata.as_ref().map(|m| serde_json::to_string(m).unwrap_or_default());

            let count = conn
                .execute(
                    "UPDATE models SET provider_id = ?1, model_identifier = ?2, display_name = ?3, enabled = ?4, local_or_remote = ?5, context_window = ?6, capabilities = ?7, metadata = ?8, updated_at = ?9 WHERE id = ?10",
                    params![
                        model.provider_id,
                        model.model_identifier,
                        model.display_name,
                        if model.enabled { 1 } else { 0 },
                        model.local_or_remote,
                        model.context_window,
                        caps_str,
                        meta_str,
                        model.updated_at,
                        model.id
                    ],
                )
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            if count == 0 {
                Err(OctrexError::NotFound {
                    resource: format!("Model with id '{}'", model.id),
                })
            } else {
                Ok(model.clone())
            }
        })
    }

    fn delete_model(&self, id: &str) -> Result<bool, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute("DELETE FROM models WHERE id = ?1", params![id])
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;
            Ok(count > 0)
        })
    }
}
