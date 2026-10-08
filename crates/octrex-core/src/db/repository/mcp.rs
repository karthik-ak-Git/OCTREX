use crate::db::manager::DatabaseManager;
use crate::db::models::McpConnectionRecord;
use crate::error::OctrexError;
use rusqlite::params;

pub trait McpConnectionRepository: Send + Sync {
    fn create_mcp_connection(
        &self,
        conn_record: &McpConnectionRecord,
    ) -> Result<McpConnectionRecord, OctrexError>;
    fn get_mcp_connection(&self, id: &str) -> Result<Option<McpConnectionRecord>, OctrexError>;
    fn list_mcp_connections(&self) -> Result<Vec<McpConnectionRecord>, OctrexError>;
    fn update_mcp_connection(
        &self,
        conn_record: &McpConnectionRecord,
    ) -> Result<McpConnectionRecord, OctrexError>;
    fn delete_mcp_connection(&self, id: &str) -> Result<bool, OctrexError>;
}

pub struct SqliteMcpConnectionRepository {
    db: DatabaseManager,
}

impl SqliteMcpConnectionRepository {
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }
}

impl McpConnectionRepository for SqliteMcpConnectionRepository {
    fn create_mcp_connection(
        &self,
        conn_record: &McpConnectionRecord,
    ) -> Result<McpConnectionRecord, OctrexError> {
        self.db.with_conn(|conn| {
            let config_str = conn_record.configuration.to_string();

            conn.execute(
                "INSERT INTO mcp_connections (id, name, server_type, configuration, enabled, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    conn_record.id,
                    conn_record.name,
                    conn_record.server_type,
                    config_str,
                    if conn_record.enabled { 1 } else { 0 },
                    conn_record.status,
                    conn_record.created_at,
                    conn_record.updated_at
                ],
            )
            .map_err(|e| OctrexError::Internal {
                message: format!("Failed to insert MCP connection: {}", e),
            })?;
            Ok(conn_record.clone())
        })
    }

    fn get_mcp_connection(&self, id: &str) -> Result<Option<McpConnectionRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, name, server_type, configuration, enabled, status, created_at, updated_at FROM mcp_connections WHERE id = ?1")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let result = stmt.query_row(params![id], |row| {
                let id_str: String = row.get(0)?;
                let name: String = row.get(1)?;
                let server_type: String = row.get(2)?;
                let config_str: String = row.get(3)?;
                let enabled_val: i32 = row.get(4)?;
                let status: String = row.get(5)?;
                let created_at: u64 = row.get(6)?;
                let updated_at: u64 = row.get(7)?;

                let configuration = serde_json::from_str(&config_str).unwrap_or_default();

                Ok(McpConnectionRecord {
                    id: id_str,
                    name,
                    server_type,
                    configuration,
                    enabled: enabled_val != 0,
                    status,
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

    fn list_mcp_connections(&self) -> Result<Vec<McpConnectionRecord>, OctrexError> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT id, name, server_type, configuration, enabled, status, created_at, updated_at FROM mcp_connections ORDER BY name ASC")
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            let rows = stmt
                .query_map([], |row| {
                    let id_str: String = row.get(0)?;
                    let name: String = row.get(1)?;
                    let server_type: String = row.get(2)?;
                    let config_str: String = row.get(3)?;
                    let enabled_val: i32 = row.get(4)?;
                    let status: String = row.get(5)?;
                    let created_at: u64 = row.get(6)?;
                    let updated_at: u64 = row.get(7)?;

                    let configuration = serde_json::from_str(&config_str).unwrap_or_default();

                    Ok(McpConnectionRecord {
                        id: id_str,
                        name,
                        server_type,
                        configuration,
                        enabled: enabled_val != 0,
                        status,
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

    fn update_mcp_connection(
        &self,
        conn_record: &McpConnectionRecord,
    ) -> Result<McpConnectionRecord, OctrexError> {
        self.db.with_conn(|conn| {
            let config_str = conn_record.configuration.to_string();

            let count = conn
                .execute(
                    "UPDATE mcp_connections SET name = ?1, server_type = ?2, configuration = ?3, enabled = ?4, status = ?5, updated_at = ?6 WHERE id = ?7",
                    params![
                        conn_record.name,
                        conn_record.server_type,
                        config_str,
                        if conn_record.enabled { 1 } else { 0 },
                        conn_record.status,
                        conn_record.updated_at,
                        conn_record.id
                    ],
                )
                .map_err(|e| OctrexError::Internal { message: e.to_string() })?;

            if count == 0 {
                Err(OctrexError::NotFound {
                    resource: format!("MCP connection with id '{}'", conn_record.id),
                })
            } else {
                Ok(conn_record.clone())
            }
        })
    }

    fn delete_mcp_connection(&self, id: &str) -> Result<bool, OctrexError> {
        self.db.with_conn(|conn| {
            let count = conn
                .execute("DELETE FROM mcp_connections WHERE id = ?1", params![id])
                .map_err(|e| OctrexError::Internal {
                    message: e.to_string(),
                })?;
            Ok(count > 0)
        })
    }
}
