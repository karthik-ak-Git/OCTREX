pub mod config;
pub mod lifecycle;
pub mod manager;
pub mod migrations;
pub mod models;
pub mod repository;

pub use config::DbConfig;
pub use lifecycle::{DatabaseState, DatabaseStatus};
pub use manager::DatabaseManager;
pub use migrations::{run_migrations, MIGRATIONS};
pub use models::*;
pub use repository::*;

#[cfg(test)]
mod tests;
