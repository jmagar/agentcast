use rusqlite::Connection;

use crate::{StoreError, StoreResult};

#[cfg(test)]
mod tests;

const MIGRATIONS: &[(i64, &str)] = &[(
    1,
    "CREATE TABLE IF NOT EXISTS catalog_snapshots (
        id TEXT PRIMARY KEY,
        created_at TEXT NOT NULL,
        payload_json TEXT NOT NULL
    )",
)];

pub fn run_migrations(conn: &Connection) -> StoreResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY)",
        [],
    )
    .map_err(|error| StoreError::Sqlite(error.to_string()))?;

    for (version, sql) in MIGRATIONS {
        let applied: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
                [version],
                |row| row.get(0),
            )
            .map_err(|error| StoreError::Sqlite(error.to_string()))?;
        if !applied {
            conn.execute_batch(sql)
                .map_err(|error| StoreError::Sqlite(error.to_string()))?;
            conn.execute(
                "INSERT INTO schema_migrations(version) VALUES (?1)",
                [version],
            )
            .map_err(|error| StoreError::Sqlite(error.to_string()))?;
        }
    }
    Ok(())
}
