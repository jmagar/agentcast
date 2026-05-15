use rusqlite::{Connection, params};

use crate::{StoreError, StoreResult};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogSnapshot {
    pub id: String,
    pub created_at: String,
    pub payload: serde_json::Value,
}

pub struct SqliteCatalogStore {
    conn: Connection,
}

impl SqliteCatalogStore {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    pub fn put_snapshot(&self, snapshot: &CatalogSnapshot) -> StoreResult<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO catalog_snapshots(id, created_at, payload_json) VALUES (?1, ?2, ?3)",
                params![snapshot.id, snapshot.created_at, snapshot.payload.to_string()],
            )
            .map_err(|error| StoreError::Sqlite(error.to_string()))?;
        Ok(())
    }

    pub fn get_snapshot(&self, id: &str) -> StoreResult<Option<CatalogSnapshot>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, created_at, payload_json FROM catalog_snapshots WHERE id = ?1")
            .map_err(|error| StoreError::Sqlite(error.to_string()))?;
        let mut rows = stmt
            .query([id])
            .map_err(|error| StoreError::Sqlite(error.to_string()))?;
        let Some(row) = rows
            .next()
            .map_err(|error| StoreError::Sqlite(error.to_string()))?
        else {
            return Ok(None);
        };
        let payload_json: String = row
            .get(2)
            .map_err(|error| StoreError::Sqlite(error.to_string()))?;
        let payload = serde_json::from_str(&payload_json)
            .map_err(|error| StoreError::Json(error.to_string()))?;
        Ok(Some(CatalogSnapshot {
            id: row
                .get(0)
                .map_err(|error| StoreError::Sqlite(error.to_string()))?,
            created_at: row
                .get(1)
                .map_err(|error| StoreError::Sqlite(error.to_string()))?,
            payload,
        }))
    }
}
