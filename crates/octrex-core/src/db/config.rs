use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub db_path: PathBuf,
    pub busy_timeout_ms: u64,
    pub wal_mode: bool,
    pub foreign_keys: bool,
}

impl DbConfig {
    pub fn default_production() -> Self {
        let base_dir = dirs::data_dir()
            .or_else(dirs::config_dir)
            .unwrap_or_else(|| PathBuf::from("."));
        let octrex_dir = base_dir.join("octrex");
        let db_path = octrex_dir.join("octrex.db");

        Self {
            db_path,
            busy_timeout_ms: 5000,
            wal_mode: true,
            foreign_keys: true,
        }
    }

    pub fn in_memory() -> Self {
        Self {
            db_path: PathBuf::from(":memory:"),
            busy_timeout_ms: 5000,
            wal_mode: false,
            foreign_keys: true,
        }
    }

    pub fn custom(path: PathBuf) -> Self {
        Self {
            db_path: path,
            busy_timeout_ms: 5000,
            wal_mode: true,
            foreign_keys: true,
        }
    }
}
