use super::*;
use crate::{open_sqlite_store, run_migrations};

#[test]
fn catalog_snapshot_round_trips() {
    let db_path = std::env::temp_dir().join(format!(
        "agentcast-store-catalog-{}-{}.db",
        std::process::id(),
        unique_suffix()
    ));
    let conn = open_sqlite_store(&db_path).unwrap();
    run_migrations(&conn).unwrap();
    let store = SqliteCatalogStore::new(conn);

    let snapshot = CatalogSnapshot {
        id: "snapshot-1".into(),
        created_at: "2026-05-12T00:00:00Z".into(),
        payload: serde_json::json!({"actions": []}),
    };

    store.put_snapshot(&snapshot).unwrap();
    let stored = store.get_snapshot("snapshot-1").unwrap().unwrap();
    assert_eq!(stored, snapshot);
    assert_eq!(stored.payload["actions"].as_array().unwrap().len(), 0);
}

#[test]
fn missing_catalog_snapshot_returns_none() {
    let db_path = std::env::temp_dir().join(format!(
        "agentcast-store-catalog-missing-{}-{}.db",
        std::process::id(),
        unique_suffix()
    ));
    let conn = open_sqlite_store(&db_path).unwrap();
    run_migrations(&conn).unwrap();
    let store = SqliteCatalogStore::new(conn);

    assert!(store.get_snapshot("missing").unwrap().is_none());
}

fn unique_suffix() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string()
}
