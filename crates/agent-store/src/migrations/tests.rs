use super::*;
use crate::open_sqlite_store;

#[test]
fn migrations_are_idempotent() {
    let db_path = std::env::temp_dir().join(format!(
        "agentcast-store-migrations-{}-{}.db",
        std::process::id(),
        unique_suffix()
    ));
    let conn = open_sqlite_store(&db_path).unwrap();

    run_migrations(&conn).unwrap();
    run_migrations(&conn).unwrap();

    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert!(version >= 1);
}

fn unique_suffix() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string()
}
