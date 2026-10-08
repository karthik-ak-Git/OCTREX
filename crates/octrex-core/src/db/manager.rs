use crate::db::config::DbConfig;
use crate::db::lifecycle::{DatabaseState, DatabaseStatus};
use crate::db::migrations::run_migrations;
use crate::error::OctrexError;
use rusqlite::Connection;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone)]
pub struct DatabaseManager {
    conn: Arc<Mutex<Option<Connection>>>,
    status: Arc<RwLock<DatabaseStatus>>,
    config: DbConfig,
}

impl DatabaseManager {
    pub fn new(config: DbConfig) -> Self {
        let status = DatabaseStatus::new(config.db_path.clone());
        Self {
            conn: Arc::new(Mutex::new(None)),
            status: Arc::new(RwLock::new(status)),
            config,
        }
    }

    pub fn initialize(&self) -> Result<(), OctrexError> {
        {
            let mut status = self.status.write().unwrap();
            status.state = DatabaseState::DatabaseInitializing;
            status.migration_status = "IN_PROGRESS".to_string();
        }

        // Ensure parent directory exists for file database
        if self.config.db_path != std::path::PathBuf::from(":memory:") {
            if let Some(parent) = self.config.db_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    let err_msg =
                        format!("Failed to create database directory {:?}: {}", parent, e);
                    self.record_error(&err_msg);
                    OctrexError::Filesystem { message: err_msg }
                })?;
            }
        }

        let mut conn = if self.config.db_path == std::path::PathBuf::from(":memory:") {
            Connection::open_in_memory()
        } else {
            Connection::open(&self.config.db_path)
        }
        .map_err(|e| {
            let err_msg = format!(
                "Failed to open SQLite database at {:?}: {}",
                self.config.db_path, e
            );
            self.record_error(&err_msg);
            OctrexError::Internal { message: err_msg }
        })?;

        // Configure connection PRAGMAs
        if self.config.foreign_keys {
            conn.execute_batch("PRAGMA foreign_keys = ON;")
                .map_err(|e| {
                    let err_msg = format!("Failed to enable foreign keys: {}", e);
                    self.record_error(&err_msg);
                    OctrexError::Internal { message: err_msg }
                })?;
        }

        if self.config.wal_mode && self.config.db_path != std::path::PathBuf::from(":memory:") {
            conn.execute_batch("PRAGMA journal_mode = WAL;")
                .map_err(|e| {
                    let err_msg = format!("Failed to enable WAL mode: {}", e);
                    self.record_error(&err_msg);
                    OctrexError::Internal { message: err_msg }
                })?;
        }

        let busy_timeout = self.config.busy_timeout_ms;
        conn.busy_timeout(std::time::Duration::from_millis(busy_timeout))
            .map_err(|e| {
                let err_msg = format!("Failed to set busy timeout: {}", e);
                self.record_error(&err_msg);
                OctrexError::Internal { message: err_msg }
            })?;

        // Run schema migrations
        let schema_version = match run_migrations(&mut conn) {
            Ok(v) => v,
            Err(e) => {
                let err_msg = format!("Database migration failed: {}", e);
                self.record_error(&err_msg);
                return Err(e);
            }
        };

        // Store active connection
        {
            let mut guard = self.conn.lock().unwrap();
            *guard = Some(conn);
        }

        // Update status to READY
        {
            let mut status = self.status.write().unwrap();
            status.state = DatabaseState::DatabaseReady;
            status.schema_version = schema_version;
            status.migration_status = "COMPLETED".to_string();
            status.last_error = None;
        }

        Ok(())
    }

    pub fn with_conn<F, R>(&self, f: F) -> Result<R, OctrexError>
    where
        F: FnOnce(&Connection) -> Result<R, OctrexError>,
    {
        let guard = self.conn.lock().map_err(|e| OctrexError::Internal {
            message: format!("Database lock poisoned: {}", e),
        })?;

        match guard.as_ref() {
            Some(conn) => f(conn),
            None => Err(OctrexError::Internal {
                message: "Database connection is not open or was closed".to_string(),
            }),
        }
    }

    pub fn with_conn_mut<F, R>(&self, f: F) -> Result<R, OctrexError>
    where
        F: FnOnce(&mut Connection) -> Result<R, OctrexError>,
    {
        let mut guard = self.conn.lock().map_err(|e| OctrexError::Internal {
            message: format!("Database lock poisoned: {}", e),
        })?;

        match guard.as_mut() {
            Some(conn) => f(conn),
            None => Err(OctrexError::Internal {
                message: "Database connection is not open or was closed".to_string(),
            }),
        }
    }

    pub fn get_status(&self) -> DatabaseStatus {
        let status = self.status.read().unwrap();
        status.clone()
    }

    pub fn shutdown(&self) {
        {
            let mut status = self.status.write().unwrap();
            status.state = DatabaseState::DatabaseShuttingDown;
        }

        {
            let mut guard = self.conn.lock().unwrap();
            *guard = None;
        }

        {
            let mut status = self.status.write().unwrap();
            status.state = DatabaseState::DatabaseClosed;
        }
    }

    fn record_error(&self, err_msg: &str) {
        let mut status = self.status.write().unwrap();
        status.state = DatabaseState::DatabaseClosed;
        status.migration_status = "FAILED".to_string();
        status.last_error = Some(err_msg.to_string());
    }
}
