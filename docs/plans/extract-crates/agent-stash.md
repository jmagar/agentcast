---
title: "agent-stash Extraction Implementation Plan"
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
  - "docs/references/acp/docs/markdown/0032-agentclientprotocol-com-protocol-file-system.md"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-stash Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `agent-stash` as the future saved-artifact and reusable-context crate for AgentCast.

**Architecture:** Stash is user-owned context and reusable material, not runtime process state. It should compose `agent-store` and `agent-protocol` models without becoming the launcher catalog or chat runtime.

**Tech Stack:** Rust 2024, serde/serde_json, thiserror, tracing, jiff, uuid.

---

## MVP Position

`agent-stash` is post-v0 unless explicitly promoted in `docs/MVP.md`. Create the boundary now so saved prompts, action templates, references, and history bundles do not get mixed into runtime or gateway crates.

## Lab Evidence Read

- `../lab/crates/lab/src/dispatch/stash/catalog.rs`
- `../lab/crates/lab/src/dispatch/stash/service.rs`
- `../lab/crates/lab/src/dispatch/stash/store.rs`
- `../lab/crates/lab/src/dispatch/stash/import.rs`
- `../lab/crates/lab/src/dispatch/stash/export.rs`
- `../lab/crates/lab/src/dispatch/stash/revision.rs`
- `../lab/crates/lab/src/dispatch/stash/providers/filesystem.rs`
- `../lab/crates/lab/src/dispatch/marketplace/stash_meta.rs`
- `../lab/crates/lab/src/mcp/services/stash.rs`
- `../lab/crates/lab-apis/src/stash.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab stash source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`.
- ACP content and filesystem-adjacent context claims are cross-checked against `docs/references/acp/docs/markdown/0028-agentclientprotocol-com-protocol-content.md` and `docs/references/acp/docs/markdown/0032-agentclientprotocol-com-protocol-file-system.md`.
- MCP resource-link assumptions are cross-checked against `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`.

Live source discovery command:

```bash
rg -n "stash|artifact|history|saved|template|reference|context" ../lab
```

## Live Lab Findings

- `dispatch/stash/*` is a real domain slice with service, store, provider, revision, import, and export boundaries.
- `marketplace/stash_meta.rs` has portable metadata, drift cache, base snapshot, path validation, and file locking behavior.
- `mcp/services/stash.rs` is an exposure layer and should not own stash behavior.

## Extraction Boundary

Extract into `agent-stash`:

- generic saved prompt/action-template patterns.
- user-owned reference bundle metadata.
- import/export shapes that do not assume Lab service workflows.
- history bundle DTOs that can be persisted by `agent-store`.

Keep out of `agent-stash`:

- runtime invocation and process management.
- gateway search/index ownership.
- API/CLI/UI rendering.
- Lab-specific service actions and credential material.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-stash/src/lib.rs` - public exports for metadata, path safety, items, revisions, import, and export.
- Create: `crates/agent-stash/src/meta.rs` - stash item metadata and drift status.
- Create: `crates/agent-stash/src/path.rs` - relative path validation.
- Create: `crates/agent-stash/src/revision.rs` - revision identity and parent tracking.
- Create: `crates/agent-stash/src/export.rs` - export manifest DTOs.
- Add sidecar tests in: `crates/agent-stash/src/{meta,path}.rs` (`#[cfg(test)] mod tests`) - metadata and path safety tests.
- Add sidecar tests in: `crates/agent-stash/src/revision.rs` (`#[cfg(test)] mod tests`) - revision serialization tests.

## Implementation Tasks

### Task 1: Define Stash Metadata And Path Safety First

**Files:**
- Create: `crates/agent-stash/src/meta.rs`
- Create: `crates/agent-stash/src/path.rs`
- Test sidecar: `crates/agent-stash/src/{meta,path}.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Read Lab stash metadata invariants.**

Run:

```bash
sed -n '1,260p' ../lab/crates/lab/src/dispatch/marketplace/stash_meta.rs
```

Expected: AgentCast preserves relative-path validation, metadata round trips, locking expectations, and drift status without Lab plugin artifact assumptions.

- [ ] **Step 2: Write failing metadata and path tests.**

Add this `#[cfg(test)] mod tests` sidecar to `crates/agent-stash/src/{meta,path}.rs`:

```rust
use super::*;

#[test]
fn stash_metadata_serializes_stable_kind() {
    let meta = StashItemMeta {
        id: "prompt-1".into(),
        kind: StashKind::PromptTemplate,
        path: "prompts/review.md".into(),
        title: "Review prompt".into(),
    };

    let value = serde_json::to_value(meta).unwrap();
    assert_eq!(value["kind"], "prompt_template");
}

#[test]
fn rejects_absolute_or_parent_paths() {
    assert!(validate_relative_stash_path("prompts/review.md").is_ok());
    assert!(validate_relative_stash_path("/etc/passwd").is_err());
    assert!(validate_relative_stash_path("../secret").is_err());
}
```

- [ ] **Step 3: Export stash modules.**

Update `crates/agent-stash/src/lib.rs`:

```rust
mod meta;
mod path;
mod revision;

pub use meta::{StashItemMeta, StashKind};
pub use path::{StashPathError, validate_relative_stash_path};
pub use revision::StashRevision;
```

- [ ] **Step 4: Implement metadata and path validation.**

Create `crates/agent-stash/src/meta.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StashItemMeta {
    pub id: String,
    pub kind: StashKind,
    pub path: String,
    pub title: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StashKind {
    PromptTemplate,
    ActionTemplate,
    ReferenceBundle,
    HistoryBundle,
}
```

Create `crates/agent-stash/src/path.rs`:

```rust
use std::path::{Component, Path};

#[derive(Debug, thiserror::Error)]
#[error("invalid stash path: {0}")]
pub struct StashPathError(String);

pub fn validate_relative_stash_path(path: &str) -> Result<(), StashPathError> {
    let path_ref = Path::new(path);
    if path_ref.is_absolute() {
        return Err(StashPathError(path.into()));
    }
    for component in path_ref.components() {
        if matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)) {
            return Err(StashPathError(path.into()));
        }
    }
    Ok(())
}
```

### Task 2: Add Revision And Export DTOs

**Files:**
- Create: `crates/agent-stash/src/revision.rs`
- Create: `crates/agent-stash/src/export.rs`
- Modify: `crates/agent-stash/src/lib.rs`
- Test sidecar: `crates/agent-stash/src/revision.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Write failing revision test.**

Add this `#[cfg(test)] mod tests` sidecar to `crates/agent-stash/src/revision.rs`:

```rust
use super::*;

#[test]
fn revision_preserves_parent_chain() {
    let revision = StashRevision {
        id: "rev-2".into(),
        parent_id: Some("rev-1".into()),
        item_id: "prompt-1".into(),
        created_at: "2026-05-12T00:00:00Z".into(),
    };

    let value = serde_json::to_value(revision).unwrap();
    assert_eq!(value["parent_id"], "rev-1");
}
```

- [ ] **Step 2: Implement revision DTO.**

Create `crates/agent-stash/src/revision.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StashRevision {
    pub id: String,
    pub parent_id: Option<String>,
    pub item_id: String,
    pub created_at: String,
}
```

- [ ] **Step 3: Verify stash tests.**

Run:

```bash
cargo test -p agent-stash
```

Expected: PASS.

### Task 3: Verify Full Stash Extraction

**Files:**
- Test sidecar: `crates/agent-stash/src/*.rs` (`#[cfg(test)] mod tests`)
- Read: `docs/plans/extract-crates/agent-stash.md`

- [ ] **Step 1: Confirm post-v0 isolation.**

Run:

```bash
rg -n "agent_stash|agent-stash" crates/agent-config crates/agent-mcp crates/agent-runtime crates/agent-gateway crates/agent-cli
```

Expected: no output until stash is promoted beyond post-v0.

- [ ] **Step 2: Run focused stash tests.**

Run:

```bash
cargo test -p agent-stash
```

Expected: DTO serialization and any storage-adapter tests pass without external services.
