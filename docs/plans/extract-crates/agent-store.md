---
title: "agent-store Extraction Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-store Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `agent-store` as the local persistence crate for AgentCast runtime, catalog, install-plan, history, and cache state.

**Architecture:** Runtime and gateway decide what should happen; `agent-store` persists durable state and exposes transaction-safe helpers. It should be usable by CLI, API, and future desktop surfaces through runtime APIs rather than becoming a surface dependency magnet.

**Tech Stack:** Rust 2024, rusqlite, directories, serde/serde_json, tokio, thiserror, tracing, jiff, uuid.

---

## MVP Position

For v0, only persist what the MCP launcher needs: config-adjacent cache metadata, discovered catalog snapshots, install-plan state, and invocation/audit records when required by contracts. Avoid broad chat/session storage until ACP is promoted.

## Lab Evidence Read

- `../lab/crates/lab/src/dispatch/marketplace/store.rs`
- `../lab/crates/lab/src/dispatch/stash/store.rs`
- `../lab/crates/lab/src/node/store.rs`
- `../lab/crates/lab/src/node/log_store.rs`
- `../lab/crates/lab-auth/src/sqlite.rs`
- `../lab/crates/lab/src/config/env_merge.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab store source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`.
- MCP catalog snapshot assumptions are cross-checked against `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`, `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`, and `docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md`.
- ACP session persistence remains post-v0 and is cross-checked against `docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md`.

Live source discovery command:

```bash
rg -n "rusqlite|sqlite|migration|persist|cache|database|state_dir|app_dir" ../lab
```

## Live Lab Findings

- `dispatch/marketplace/store.rs` has the strongest general SQLite cache pattern: migrations, restrictive permissions, pagination, latest flags, metadata records, and sync stats.
- `lab-auth/src/sqlite.rs` has auth-specific persistence that should not be pulled into generic store directly.
- `node/store.rs` and `node/log_store.rs` are post-v0 fleet evidence.

## Extraction Boundary

Extract into `agent-store`:

- app data directory persistence patterns that are generic.
- SQLite connection and migration patterns.
- store traits for catalog snapshots, install plans, runtime health history, and invocation records.
- transaction helpers and serialization boundaries.
- fixture-based tests proving schema compatibility and migration behavior.

Keep out of `agent-store`:

- runtime process supervision.
- gateway routing/collision policy.
- API response shaping.
- CLI output rendering.
- ACP chat/session persistence until ACP is promoted.
- Lab-specific service inventory, credentials, and appdata extraction.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-store/src/lib.rs` - public exports for SQLite, migrations, and store traits.
- Create: `crates/agent-store/src/error.rs` - store error type and stable kind strings.
- Create: `crates/agent-store/src/sqlite.rs` - SQLite connection opener and file-permission setup.
- Create: `crates/agent-store/src/migrations.rs` - migration list and idempotent runner.
- Create: `crates/agent-store/src/catalog.rs` - catalog snapshot store trait and SQLite implementation.
- Add source-side test sidecars for: `crates/agent-store/src/migrations.rs` - migration idempotence tests.
- Add source-side test sidecars for: `crates/agent-store/src/catalog.rs` - catalog snapshot round-trip tests.

## Implementation Tasks

### Task 1: Extract A Generic SQLite Migration Harness

**Files:**
- Create: `crates/agent-store/src/sqlite.rs`
- Create: `crates/agent-store/src/migrations.rs`
- Test sidecar: `crates/agent-store/src/migrations.rs`

- [ ] **Step 1: Read the Lab registry store migration path.**

Run:

```bash
rg -n "pub struct RegistryStore|fn migrate|set_restrictive_permissions|migration_is_idempotent" ../lab/crates/lab/src/dispatch/marketplace/store.rs
```

Expected: AgentCast store tests cover open, migrate, idempotence, and restrictive permissions without registry-specific schema leaking into the base store harness.

- [ ] **Step 2: Write failing migration tests.**

Create a source-side test sidecar next to `crates/agent-store/src/migrations.rs` with:

```rust
use super::*;

#[test]
fn migrations_are_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("agentcast.db");
    let conn = open_sqlite_store(&db_path).unwrap();

    run_migrations(&conn).unwrap();
    run_migrations(&conn).unwrap();

    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| row.get(0))
        .unwrap();
    assert!(version >= 1);
}
```

- [ ] **Step 3: Export SQLite modules.**

Update `crates/agent-store/src/lib.rs`:

```rust
mod error;
mod migrations;
mod sqlite;

pub use error::{StoreError, StoreResult};
pub use migrations::run_migrations;
pub use sqlite::open_sqlite_store;
```

- [ ] **Step 4: Implement error, open, and migrations.**

Create `crates/agent-store/src/error.rs`:

```rust
use thiserror::Error;

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
```

Create `crates/agent-store/src/sqlite.rs`:

```rust
use std::path::Path;

use rusqlite::Connection;

use crate::StoreResult;

pub fn open_sqlite_store(path: &Path) -> StoreResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(Connection::open(path)?)
}
```

Create `crates/agent-store/src/migrations.rs`:

```rust
use rusqlite::Connection;

use crate::StoreResult;

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
    )?;
    for (version, sql) in MIGRATIONS {
        let applied: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
            [version],
            |row| row.get(0),
        )?;
        if !applied {
            conn.execute_batch(sql)?;
            conn.execute("INSERT INTO schema_migrations(version) VALUES (?1)", [version])?;
        }
    }
    Ok(())
}
```

### Task 2: Add Catalog Snapshot Store

**Files:**
- Create: `crates/agent-store/src/catalog.rs`
- Modify: `crates/agent-store/src/lib.rs`
- Test sidecar: `crates/agent-store/src/catalog.rs`

- [ ] **Step 1: Write failing catalog round-trip test.**

Create a source-side test sidecar next to `crates/agent-store/src/catalog.rs` with:

```rust
use super::*;

#[test]
fn catalog_snapshot_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let conn = open_sqlite_store(&dir.path().join("agentcast.db")).unwrap();
    run_migrations(&conn).unwrap();
    let store = SqliteCatalogStore::new(conn);

    let snapshot = CatalogSnapshot {
        id: "snapshot-1".into(),
        created_at: "2026-05-12T00:00:00Z".into(),
        payload: serde_json::json!({"actions": []}),
    };

    store.put_snapshot(&snapshot).unwrap();
    assert_eq!(store.get_snapshot("snapshot-1").unwrap().unwrap().payload["actions"].as_array().unwrap().len(), 0);
}
```

- [ ] **Step 2: Export catalog store types.**

Update `crates/agent-store/src/lib.rs`:

```rust
mod catalog;
pub use catalog::{CatalogSnapshot, SqliteCatalogStore};
```

- [ ] **Step 3: Implement catalog store.**

Create `crates/agent-store/src/catalog.rs`:

```rust
use rusqlite::{Connection, params};

use crate::StoreResult;

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
        self.conn.execute(
            "INSERT OR REPLACE INTO catalog_snapshots(id, created_at, payload_json) VALUES (?1, ?2, ?3)",
            params![snapshot.id, snapshot.created_at, snapshot.payload.to_string()],
        )?;
        Ok(())
    }

    pub fn get_snapshot(&self, id: &str) -> StoreResult<Option<CatalogSnapshot>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, payload_json FROM catalog_snapshots WHERE id = ?1",
        )?;
        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            let payload_json: String = row.get(2)?;
            Ok(Some(CatalogSnapshot {
                id: row.get(0)?,
                created_at: row.get(1)?,
                payload: serde_json::from_str(&payload_json).unwrap_or(serde_json::Value::Null),
            }))
        } else {
            Ok(None)
        }
    }
}
```

### Task 3: Verify Full Store Extraction

**Files:**
- Test sidecar: `crates/agent-store/src/*.rs`
- Read: `docs/plans/extract-crates/agent-store.md`

- [ ] **Step 1: Run focused store tests.**

Run:

```bash
cargo nextest run -p agent-store
```

Expected: migrations and store round trips pass using temp directories/databases only.

- [ ] **Step 2: Scan for Lab-specific schema leakage.**

Run:

```bash
rg -n "Plex|Sonarr|Radarr|Unraid|Gotify|LAB_|\\.lab|node|fleet|auth_session" crates/agent-store
```

Expected: no output for v0 store code.
