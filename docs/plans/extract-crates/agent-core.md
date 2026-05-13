---
title: "agent-core Extraction Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/acp/docs/markdown/0028-agentclientprotocol-com-protocol-content.md"
  - "docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-core Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract shared AgentCast primitives that every crate can depend on without creating domain cycles.

**Architecture:** `agent-core` owns IDs, timestamps, error primitives, action metadata, and small shared utilities. It must stay protocol-light and must not depend on MCP, ACP, Axum, Clap, or runtime crates.

**Tech Stack:** Rust 2024, serde, serde_json, thiserror, uuid, jiff, url.

---

## MVP Position

Core should be implemented before higher crates so all extracted behavior uses one error and metadata vocabulary.

## Lab Source Files

- `../lab/crates/lab-apis/src/core.rs`
- `../lab/crates/lab/src/dispatch/error.rs`
- `../lab/crates/lab/src/dispatch/helpers.rs`
- `../lab/crates/lab/src/registry.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab core source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`.
- Shared MCP action/tool/schema assumptions are cross-checked against `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md` and `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`.
- Shared ACP content/tool-call assumptions are cross-checked against `docs/references/acp/docs/markdown/0028-agentclientprotocol-com-protocol-content.md` and `docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md`.

## Live Lab Findings

- `lab-apis/src/core.rs` is the source for shared action metadata and service status shapes.
- `dispatch/error.rs` proves error kind strings and custom serialization are shared across CLI/API/MCP.
- `dispatch/helpers.rs` has reusable JSON parameter helpers and path safety helpers; do not extract its service client `InstancePool` into core.

## Extraction Boundary

Extract:

- action metadata shapes.
- stable ID and timestamp helper patterns.
- typed error categories that are not tied to HTTP or CLI.
- small JSON parameter helpers when they are shared across surfaces.

Leave behind:

- Lab plugin metadata tied to homelab service catalogs.
- surface-specific error envelopes.
- registry composition that depends on Lab service modules.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-core/src/lib.rs` - public exports for action, error, ID, timestamp, and JSON helpers.
- Create: `crates/agent-core/src/action.rs` - shared action metadata, risk labels, and category primitives.
- Create: `crates/agent-core/src/error.rs` - protocol-neutral error kind/category model.
- Create: `crates/agent-core/src/id.rs` - stable ID helpers for actions and upstreams.
- Create: `crates/agent-core/src/time.rs` - timestamp wrapper helpers.
- Create: `crates/agent-core/src/json.rs` - small JSON object extraction helpers.
- Add source-side test sidecars for: `crates/agent-core/src/action.rs` - action serialization tests.
- Add source-side test sidecars for: `crates/agent-core/src/error.rs` - stable error kind tests.
- Add source-side test sidecars for: `crates/agent-core/src/lib.rs` - dependency direction test.

## Implementation Tasks

### Task 1: Define Shared Action And Error Types

**Files:**
- Modify: `crates/agent-core/src/lib.rs`
- Create: `crates/agent-core/src/action.rs`
- Create: `crates/agent-core/src/error.rs`
- Create: `crates/agent-core/src/id.rs`
- Test sidecar: `crates/agent-core/src/action.rs`

- [ ] **Step 1: Inspect Lab action metadata.**

Run:

```bash
sed -n '1,220p' ../lab/crates/lab-apis/src/core.rs
```

Expected: action metadata and param metadata candidates are identified.

- [ ] **Step 2: Write failing action serialization tests.**

Create a source-side test sidecar next to `crates/agent-core/src/action.rs` with:

```rust
use super::*;

#[test]
fn action_metadata_serializes_stable_kind_strings() {
    let metadata = ActionMetadata {
        action_id: "mcp:filesystem:tool:read_file".into(),
        title: "Read file".into(),
        description: Some("Read a local file".into()),
        category: Some("filesystem".into()),
        risk: ActionRisk::ReadOnly,
    };

    let value = serde_json::to_value(metadata).unwrap();
    assert_eq!(value["action_id"], "mcp:filesystem:tool:read_file");
    assert_eq!(value["risk"], "read_only");
}
```

Run:

```bash
cargo nextest run -p agent-core action
```

Expected: FAIL because `ActionMetadata` and `ActionRisk` do not exist yet.

- [ ] **Step 3: Export core modules.**

Update `crates/agent-core/src/lib.rs`:

```rust
mod action;
mod error;
mod id;
mod json;
mod time;

pub use action::{ActionMetadata, ActionRisk};
pub use error::{AgentErrorKind, CoreError, CoreResult};
pub use id::{normalize_id_segment, scoped_action_id};
pub use json::expect_json_object;
pub use time::now_timestamp;
```

- [ ] **Step 4: Implement action metadata.**

Create `crates/agent-core/src/action.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionMetadata {
    pub action_id: String,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub risk: ActionRisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionRisk {
    ReadOnly,
    Mutating,
    Destructive,
    Unknown,
}
```

- [ ] **Step 5: Implement ID helpers.**

Create `crates/agent-core/src/id.rs`:

```rust
pub fn normalize_id_segment(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

pub fn scoped_action_id(scope: &str, upstream_id: &str, action: &str) -> String {
    format!(
        "{}.{}.{}",
        normalize_id_segment(scope),
        normalize_id_segment(upstream_id),
        normalize_id_segment(action)
    )
}
```

- [ ] **Step 6: Verify action tests.**

Run:

```bash
cargo nextest run -p agent-core action
```

Expected: PASS.

### Task 2: Port Stable Error-Kind Tests

**Files:**
- Create: `crates/agent-core/src/error.rs`
- Create: `crates/agent-core/src/json.rs`
- Test sidecar: `crates/agent-core/src/error.rs`

- [ ] **Step 1: Read Lab stable error-kind sources.**

Run:

```bash
rg -n "Serialize for ToolError|pub const fn kind|canonical_kind" ../lab/crates/lab/src/dispatch/error.rs ../lab/crates/lab/src/mcp/error.rs
```

Expected: AgentCast error kind tests use AgentCast names and omit Lab service variants.

- [ ] **Step 2: Write failing error tests.**

Create a source-side test sidecar next to `crates/agent-core/src/error.rs` with:

```rust
use super::*;
use serde_json::json;

#[test]
fn error_kind_serializes_as_stable_string() {
    let kind = AgentErrorKind::InvalidParams;
    assert_eq!(serde_json::to_value(kind).unwrap(), "invalid_params");
}

#[test]
fn core_error_exposes_stable_kind() {
    let error = CoreError::invalid_params("missing path");
    assert_eq!(error.kind(), AgentErrorKind::InvalidParams);
    assert_eq!(error.kind().as_str(), "invalid_params");
}

#[test]
fn json_helper_rejects_non_object_params() {
    let err = expect_json_object(&json!(["not", "object"])).unwrap_err();
    assert_eq!(err.kind(), AgentErrorKind::InvalidParams);
}
```

- [ ] **Step 3: Implement error kinds and core error.**

Create `crates/agent-core/src/error.rs`:

```rust
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentErrorKind {
    InvalidParams,
    NotFound,
    Conflict,
    Internal,
}

impl AgentErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidParams => "invalid_params",
            Self::NotFound => "not_found",
            Self::Conflict => "conflict",
            Self::Internal => "internal",
        }
    }
}

#[derive(Debug, Error)]
#[error("{kind:?}: {message}")]
pub struct CoreError {
    kind: AgentErrorKind,
    message: String,
}

impl CoreError {
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            kind: AgentErrorKind::InvalidParams,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> AgentErrorKind {
        self.kind
    }
}
```

- [ ] **Step 4: Implement JSON helper.**

Create `crates/agent-core/src/json.rs`:

```rust
use serde_json::{Map, Value};

use crate::{CoreError, CoreResult};

pub fn expect_json_object(value: &Value) -> CoreResult<&Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| CoreError::invalid_params("expected JSON object"))
}
```

- [ ] **Step 5: Add timestamp helper.**

Create `crates/agent-core/src/time.rs`:

```rust
pub fn now_timestamp() -> jiff::Timestamp {
    jiff::Timestamp::now()
}
```

- [ ] **Step 6: Verify error tests.**

Run:

```bash
cargo nextest run -p agent-core error
```

Expected: PASS.

### Task 3: Verify Dependency Direction

**Files:**
- Read: `crates/*/Cargo.toml`
- Test sidecar: `crates/agent-core/src/lib.rs`

- [ ] **Step 1: Check that `agent-core` has no AgentCast crate dependencies.**

Run:

```bash
sed -n '1,120p' crates/agent-core/Cargo.toml
```

Expected: `agent-core` depends only on workspace external crates.

- [ ] **Step 2: Add dependency boundary test.**

Create a source-side test sidecar next to `crates/agent-core/src/lib.rs` with:

```rust
#[test]
fn core_cargo_toml_has_no_agent_crate_dependencies() {
    let manifest = std::fs::read_to_string("Cargo.toml").unwrap();
    for forbidden in [
        "agent-protocol",
        "agent-mcp",
        "agent-gateway",
        "agent-runtime",
        "agent-api",
    ] {
        assert!(!manifest.contains(forbidden), "{forbidden} must not be a core dependency");
    }
}
```

- [ ] **Step 3: Verify core tests.**

Run:

```bash
cargo nextest run -p agent-core
```

Expected: core tests pass.

- [ ] **Step 4: Commit the core extraction slice.**

Run:

```bash
git add crates/agent-core docs/plans/extract-crates/agent-core.md
git commit -m "feat(core): extract shared primitives"
```

Expected: commit contains only `agent-core` implementation, tests, and this plan if executing this slice alone.
