use std::path::Path;

use rusqlite::Connection;

use crate::{StoreError, StoreResult};

pub fn open_sqlite_store(path: &Path) -> StoreResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| StoreError::Io(error.to_string()))?;
    }
    let conn = Connection::open(path).map_err(|error| StoreError::Sqlite(error.to_string()))?;
    restrict_permissions(path)?;
    Ok(conn)
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> StoreResult<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path).map_err(|error| StoreError::Io(error.to_string()))?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o600);
    std::fs::set_permissions(path, permissions).map_err(|error| StoreError::Io(error.to_string()))
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> StoreResult<()> {
    Ok(())
}
