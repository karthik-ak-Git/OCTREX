use crate::db::manager::DatabaseManager;
use crate::db::models::SettingRecord;
use crate::error::OctrexError;
use rusqlite::params;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait SettingsRepository: Send + Sync {
    fn set_setting(
        &self,
        category: &str,
        key: &str,
        value: &str,
    ) -> Result<SettingRecord, OctrexError>;
    fn get_setting(&self, category: &str, key: &str) -> Result<Option<SettingRecord>, OctrexError>;
    fn list_settings_by_category(&self, category: &str) -> Result<Vec<SettingRecord>, OctrexError>;
    fn list_all_settings(&self) -> Result<Vec<SettingRecord>, OctrexError>;
    fn delete_setting(&self, category: &str, key: &str) -> Result<bool, OctrexError>;
}

pub struct SqliteSettingsRepository {
    db: DatabaseManager,
}

impl SqliteSettingsRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
}

impl SettingsRepository for SqliteSettingsRepository {
    fn set_setting(
        &self,
        category: &str,
        key: &str,
        value: &str,
    ) -> Result<SettingRecord, OctrexError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let record = SettingRecord {
            category: category.to_string(),
            key: key.to_string(),
            value: value.to_string(),
            updated_at: now,
        };

        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO settings (category, key, value, updated_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(category, key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![category, key, value, now],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to set setting ({}, {}): {}", category, key, e),
            })?;
            Ok(record)
        })
    }

    fn get_setting(&self, category: &str, key: &str) -> Result<Option<SettingRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT category, key, value, updated_at FROM settings WHERE category = ?1 AND key = ?2")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let result = stmt.query_row(params![category, key], |row| {
                let cat: String = row.get(0)?;
                let k: String = row.get(1)?;
                let val: String = row.get(2)?;
                let updated_at: u64 = row.get(3)?;

                Ok(SettingRecord {
                    category: cat,
                    key: k,
                    value: val,
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

    fn list_settings_by_category(&self, category: &str) -> Result<Vec<SettingRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT category, key, value, updated_at FROM settings WHERE category = ?1 ORDER BY key ASC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map(params![category], |row| {
                    let cat: String = row.get(0)?;
                    let k: String = row.get(1)?;
                    let val: String = row.get(2)?;
                    let updated_at: u64 = row.get(3)?;

                    Ok(SettingRecord {
                        category: cat,
                        key: k,
                        value: val,
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

    fn list_all_settings(&self) -> Result<Vec<SettingRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT category, key, value, updated_at FROM settings ORDER BY category, key ASC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map([], |row| {
                    let cat: String = row.get(0)?;
                    let k: String = row.get(1)?;
                    let val: String = row.get(2)?;
                    let updated_at: u64 = row.get(3)?;

                    Ok(SettingRecord {
                        category: cat,
                        key: k,
                        value: val,
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

    fn delete_setting(&self, category: &str, key: &str) -> Result<bool, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute(
                    "DELETE FROM settings WHERE category = ?1 AND key = ?2",
                    params![category, key],
                )
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;
            Ok(count > 0)
        })
    }
}
