use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DatabaseState {
    DatabaseUninitialized,
    DatabaseInitializing,
    DatabaseReady,
    DatabaseShuttingDown,
    DatabaseClosed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStatus {
    pub state: DatabaseState,
    pub path: PathBuf,
    pub schema_version: u32,
    pub migration_status: String,
    pub last_error: Option<String>,
}

impl DatabaseStatus {
    pub fn new(path: PathBuf) -> Self {
        Self {
            state: DatabaseState::DatabaseUninitialized,
            path,
            schema_version: 0,
            migration_status: "NOT_STARTED".to_string(),
            last_error: None,
        }
    }
}
