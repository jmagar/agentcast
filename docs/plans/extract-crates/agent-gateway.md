---
title: "agent-gateway Extraction Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/acp/docs/markdown/0022-agentclientprotocol-com-protocol-overview.md"
  - "docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md"
  - "docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md"
  - "docs/references/mcp/docs/markdown/0185-modelcontextprotocol-io-docs-learn-architecture.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-gateway Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract Lab's gateway patterns for catalog merge, collision handling, exposure filtering, and invocation routing into AgentCast.

**Architecture:** `agent-gateway` owns capability projection and routing across MCP and post-v0 ACP surfaces. MCP protocol lifecycle remains in `agent-mcp`; local process/session lifecycle remains in `agent-runtime`.

**Tech Stack:** Rust 2024, RMCP, serde_json, Tokio, AgentCast protocol models.

---

## MVP Position

Gateway is required for the MCP launcher MVP after config and MCP lifecycle are in place.

## Lab Source Files

- `../lab/crates/lab/src/dispatch/gateway/catalog.rs`
- `../lab/crates/lab/src/dispatch/gateway/client.rs`
- `../lab/crates/lab/src/dispatch/gateway/config.rs`
- `../lab/crates/lab/src/dispatch/gateway/dispatch.rs`
- `../lab/crates/lab/src/dispatch/gateway/index.rs`
- `../lab/crates/lab/src/dispatch/gateway/params.rs`
- `../lab/crates/lab/src/dispatch/gateway/projection.rs`
- `../lab/crates/lab/src/dispatch/gateway/runtime.rs`
- `../lab/crates/lab/src/dispatch/gateway/service_catalog.rs`
- `../lab/crates/lab/src/dispatch/gateway/types.rs`
- `../lab/crates/lab/src/dispatch/gateway/virtual_servers.rs`
- `../lab/crates/lab/src/dispatch/gateway/view_models.rs`
- `../lab/crates/lab/src/dispatch/gateway/protected_routes.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab gateway source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`; the snapshot exposes GatewayManager runtime impls in `crates/lab/src/dispatch/gateway/runtime.rs` and does not include a `crates/lab/src/dispatch/gateway/manager.rs` file entry.
- MCP capability projection, routing, tools, resources, prompts, lifecycle, transport, and OAuth boundary claims are cross-checked against `docs/references/mcp/docs/markdown/0185-modelcontextprotocol-io-docs-learn-architecture.md`, `docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md`, `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`, `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`, `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`, `docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md`, and `docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md`.
- ACP gateway hand-off claims are post-v0 and are cross-checked against `docs/references/acp/docs/markdown/0022-agentclientprotocol-com-protocol-overview.md` and `docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md`.

## Live Lab Findings

- `index.rs` contains `IndexedTool`, `ToolIndex`, `SearchHit`, scoring, and catalog hashing. Search should move to `agent-search`; gateway should expose searchable documents and consume ranked hits.
- The Lab `manager` module combines catalog diffing, reload, search rebuild, runtime status, config persistence, OAuth, and cleanup across the gateway module; in the reference snapshot, runtime-specific GatewayManager impls are in `runtime.rs`. AgentCast should split these responsibilities instead of copying the manager module whole.
- `projection.rs` contains useful sanitization and redaction behavior for server/tool views.
- `config.rs` belongs mostly to `agent-config`; gateway should depend on validated config instead of rewriting config mutation.

## Extraction Boundary

Extract:

- discovered tool/resource/prompt projection.
- normalized launcher action catalog.
- collision strategy.
- exposure filtering.
- deterministic invocation routing.
- gateway health aggregation.

Leave behind:

- Lab service catalog assumptions.
- OAuth lifecycle until MCP OAuth is explicitly in scope.
- ACP-to-gateway bridge until ACP post-v0 work starts.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-gateway/src/lib.rs` - public exports for catalog, collision, router, invocation, projection, and errors.
- Create: `crates/agent-gateway/src/error.rs` - gateway error type and stable kind strings.
- Create: `crates/agent-gateway/src/catalog.rs` - launcher action catalog and merge behavior.
- Create: `crates/agent-gateway/src/collision.rs` - action ID collision resolution policy.
- Create: `crates/agent-gateway/src/projection.rs` - MCP metadata to launcher action projection.
- Create: `crates/agent-gateway/src/router.rs` - deterministic action ID to upstream/tool routing table.
- Create: `crates/agent-gateway/src/invoke.rs` - invocation request/result envelopes.
- Add sidecar tests in: `crates/agent-gateway/src/catalog.rs` (`#[cfg(test)] mod tests`) - merge and collision tests.
- Add sidecar tests in: `crates/agent-gateway/src/projection.rs` (`#[cfg(test)] mod tests`) - MCP tool projection tests.
- Add sidecar tests in: `crates/agent-gateway/src/router.rs` (`#[cfg(test)] mod tests`) - deterministic routing tests.
- Add sidecar tests in: `crates/agent-gateway/src/catalog.rs` (`#[cfg(test)] mod tests`) - search handoff tests with `agent-search`.

## Implementation Tasks

### Task 1: Define Launcher Catalog Merge Contract

**Files:**
- Modify: `crates/agent-gateway/src/lib.rs`
- Create: `crates/agent-gateway/src/error.rs`
- Create: `crates/agent-gateway/src/catalog.rs`
- Create: `crates/agent-gateway/src/collision.rs`
- Test sidecar: `crates/agent-gateway/src/catalog.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Inspect Lab projection and index code.**

Run:

```bash
sed -n '1,260p' ../lab/crates/lab/src/dispatch/gateway/projection.rs
sed -n '1,260p' ../lab/crates/lab/src/dispatch/gateway/index.rs
```

Expected: reusable projection and indexing behavior is identified.

- [ ] **Step 2: Write failing catalog merge sidecar tests.**

Add this `#[cfg(test)] mod tests` sidecar to `crates/agent-gateway/src/catalog.rs`:

```rust
use super::*;

fn action(action_id: &str, upstream_id: &str, tool_name: &str) -> GatewayAction {
    GatewayAction {
        action_id: action_id.into(),
        upstream_id: upstream_id.into(),
        tool_name: tool_name.into(),
        title: tool_name.into(),
        description: None,
        risk: RiskLevel::Unknown,
        input_schema: serde_json::json!({"type": "object"}),
    }
}

#[test]
fn merge_keeps_stable_action_ids_for_unique_tools() {
    let mut catalog = ActionCatalog::default();
    catalog.merge([action("mcp:filesystem:tool:read_file", "filesystem", "read_file")]);

    let stored = catalog.get("mcp:filesystem:tool:read_file").expect("action exists");
    assert_eq!(stored.upstream_id, "filesystem");
    assert_eq!(stored.tool_name, "read_file");
}

#[test]
fn merge_rejects_collision_with_different_route() {
    let mut catalog = ActionCatalog::default();
    catalog.merge([action("mcp:filesystem:tool:read", "filesystem", "read")]);
    let report = catalog.merge([action("mcp:filesystem:tool:read", "other", "read")]);

    assert_eq!(report.collisions.len(), 1);
    assert_eq!(catalog.get("mcp:filesystem:tool:read").unwrap().upstream_id, "filesystem");
}
```

Run:

```bash
cargo test -p agent-gateway catalog_merge
```

Expected: FAIL because `ActionCatalog`, `GatewayAction`, and `RiskLevel` do not exist yet.

- [ ] **Step 3: Export gateway modules and error type.**

Update `crates/agent-gateway/src/lib.rs`:

```rust
mod catalog;
mod collision;
mod error;

pub use catalog::{ActionCatalog, CatalogMergeReport, GatewayAction, RiskLevel};
pub use collision::CatalogCollision;
pub use error::{GatewayError, GatewayResult};
```

Create `crates/agent-gateway/src/error.rs`:

```rust
use thiserror::Error;

pub type GatewayResult<T> = Result<T, GatewayError>;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("unknown action: {0}")]
    UnknownAction(String),
    #[error("action collision: {0}")]
    Collision(String),
    #[error("invocation failed: {0}")]
    Invocation(String),
}

impl GatewayError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::UnknownAction(_) => "unknown_action",
            Self::Collision(_) => "action_collision",
            Self::Invocation(_) => "invocation_failed",
        }
    }
}
```

- [ ] **Step 4: Implement catalog and collision report.**

Create `crates/agent-gateway/src/collision.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogCollision {
    pub action_id: String,
    pub existing_route: String,
    pub incoming_route: String,
}
```

Create `crates/agent-gateway/src/catalog.rs`:

```rust
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::CatalogCollision;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    ReadOnly,
    Mutating,
    Destructive,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayAction {
    pub action_id: String,
    pub upstream_id: String,
    pub tool_name: String,
    pub title: String,
    pub description: Option<String>,
    pub risk: RiskLevel,
    pub input_schema: Value,
}

impl GatewayAction {
    pub fn route_key(&self) -> String {
        format!("{}::{}", self.upstream_id, self.tool_name)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CatalogMergeReport {
    pub inserted: Vec<String>,
    pub updated: Vec<String>,
    pub collisions: Vec<CatalogCollision>,
}

#[derive(Debug, Default, Clone)]
pub struct ActionCatalog {
    actions: BTreeMap<String, GatewayAction>,
}

impl ActionCatalog {
    pub fn merge(&mut self, actions: impl IntoIterator<Item = GatewayAction>) -> CatalogMergeReport {
        let mut report = CatalogMergeReport::default();
        for action in actions {
            match self.actions.get(&action.action_id) {
                Some(existing) if existing.route_key() != action.route_key() => {
                    report.collisions.push(CatalogCollision {
                        action_id: action.action_id,
                        existing_route: existing.route_key(),
                        incoming_route: action.route_key(),
                    });
                }
                Some(_) => {
                    report.updated.push(action.action_id.clone());
                    self.actions.insert(action.action_id.clone(), action);
                }
                None => {
                    report.inserted.push(action.action_id.clone());
                    self.actions.insert(action.action_id.clone(), action);
                }
            }
        }
        report
    }

    pub fn get(&self, action_id: &str) -> Option<&GatewayAction> {
        self.actions.get(action_id)
    }
}
```

- [ ] **Step 5: Verify catalog merge tests.**

Run:

```bash
cargo test -p agent-gateway catalog_merge
```

Expected: PASS.

### Task 2: Project MCP Tools Into Launcher Actions

**Files:**
- Create: `crates/agent-gateway/src/projection.rs`
- Modify: `crates/agent-gateway/src/lib.rs`
- Test sidecar: `crates/agent-gateway/src/projection.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Write failing projection sidecar test.**

Add this `#[cfg(test)] mod tests` sidecar to `crates/agent-gateway/src/projection.rs`:

```rust
use super::*;
use agent_mcp::RawMcpTool;
use serde_json::json;

#[test]
fn project_mcp_tool_uses_annotations_for_risk() {
    let raw = RawMcpTool {
        name: "delete_file".into(),
        title: Some("Delete file".into()),
        description: Some("Delete a file".into()),
        input_schema: json!({"type": "object"}),
        annotations: json!({"destructiveHint": true}),
    };

    let action = project_mcp_tool("filesystem", raw);
    assert_eq!(action.action_id, "mcp:filesystem:tool:delete_file");
    assert_eq!(action.risk, RiskLevel::Destructive);
    assert_eq!(action.title, "Delete file");
}
```

- [ ] **Step 2: Export projection helper.**

Update `crates/agent-gateway/src/lib.rs`:

```rust
mod projection;
pub use projection::project_mcp_tool;
```

- [ ] **Step 3: Implement projection.**

Create `crates/agent-gateway/src/projection.rs`:

```rust
use agent_mcp::RawMcpTool;

use crate::{GatewayAction, RiskLevel};

pub fn project_mcp_tool(upstream_id: &str, tool: RawMcpTool) -> GatewayAction {
    let destructive = tool
        .annotations
        .get("destructiveHint")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let read_only = tool
        .annotations
        .get("readOnlyHint")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);

    let risk = if destructive {
        RiskLevel::Destructive
    } else if read_only {
        RiskLevel::ReadOnly
    } else {
        RiskLevel::Unknown
    };

    GatewayAction {
        action_id: format!("mcp:{upstream_id}:tool:{}", tool.name),
        upstream_id: upstream_id.into(),
        tool_name: tool.name.clone(),
        title: tool.title.unwrap_or(tool.name),
        description: tool.description,
        risk,
        input_schema: tool.input_schema,
    }
}
```

- [ ] **Step 4: Verify projection tests.**

Run:

```bash
cargo test -p agent-gateway projection
```

Expected: PASS.

### Task 3: Route Deterministic Invocations

**Files:**
- Create: `crates/agent-gateway/src/router.rs`
- Create: `crates/agent-gateway/src/invoke.rs`
- Test sidecar: `crates/agent-gateway/src/router.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Inspect Lab gateway manager.**

Run:

```bash
sed -n '1,320p' ../lab/crates/lab/src/dispatch/gateway/runtime.rs
```

Expected: manager responsibilities are split between AgentCast gateway and MCP crates.

- [ ] **Step 2: Write failing routing sidecar test.**

Add this `#[cfg(test)] mod tests` sidecar to `crates/agent-gateway/src/router.rs`:

```rust
use super::*;

#[test]
fn router_resolves_action_id_to_upstream_tool_route() {
    let mut catalog = ActionCatalog::default();
    catalog.merge([GatewayAction {
        action_id: "mcp:filesystem:tool:read_file".into(),
        upstream_id: "filesystem".into(),
        tool_name: "read_file".into(),
        title: "Read file".into(),
        description: None,
        risk: RiskLevel::ReadOnly,
        input_schema: serde_json::json!({"type": "object"}),
    }]);

    let router = GatewayRouter::new(catalog);
    let route = router
        .resolve(&InvokeRequest {
            action_id: "mcp:filesystem:tool:read_file".into(),
            params: serde_json::json!({"path": "/tmp/a.txt"}),
        })
        .expect("route exists");

    assert_eq!(route.upstream_id, "filesystem");
    assert_eq!(route.tool_name, "read_file");
}
```

- [ ] **Step 3: Export router and invocation types.**

Update `crates/agent-gateway/src/lib.rs`:

```rust
mod invoke;
mod router;

pub use invoke::{InvokeRequest, InvokeResult};
pub use router::{GatewayRoute, GatewayRouter};
```

- [ ] **Step 4: Implement invocation envelopes.**

Create `crates/agent-gateway/src/invoke.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvokeRequest {
    pub action_id: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvokeResult {
    pub action_id: String,
    #[serde(default)]
    pub output: serde_json::Value,
}
```

- [ ] **Step 5: Implement deterministic router.**

Create `crates/agent-gateway/src/router.rs`:

```rust
use crate::{ActionCatalog, GatewayError, GatewayResult, InvokeRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayRoute {
    pub upstream_id: String,
    pub tool_name: String,
}

#[derive(Debug, Clone)]
pub struct GatewayRouter {
    catalog: ActionCatalog,
}

impl GatewayRouter {
    pub fn new(catalog: ActionCatalog) -> Self {
        Self { catalog }
    }

    pub fn resolve(&self, request: &InvokeRequest) -> GatewayResult<GatewayRoute> {
        let action = self
            .catalog
            .get(&request.action_id)
            .ok_or_else(|| GatewayError::UnknownAction(request.action_id.clone()))?;

        Ok(GatewayRoute {
            upstream_id: action.upstream_id.clone(),
            tool_name: action.tool_name.clone(),
        })
    }
}
```

- [ ] **Step 6: Verify routing tests.**

Run:

```bash
cargo test -p agent-gateway invocation_routing
```

Expected: PASS.

### Task 4: Split Search Out Of Gateway

**Files:**
- Modify: `crates/agent-gateway/src/catalog.rs`
- Modify: `crates/agent-search/src/lib.rs`
- Test sidecar: `crates/agent-gateway/src/catalog.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Use Lab tool index as source evidence.**

Run:

```bash
sed -n '1,220p' ../lab/crates/lab/src/dispatch/gateway/index.rs
```

Expected: `agent-gateway` delegates ranking to `agent-search` and keeps routing/collision policy.

- [ ] **Step 2: Write failing search handoff sidecar test.**

Add this `#[cfg(test)] mod tests` sidecar to `crates/agent-gateway/src/catalog.rs`:

```rust
use super::*;

#[test]
fn catalog_exports_search_documents_without_ranking() {
    let mut catalog = ActionCatalog::default();
    catalog.merge([GatewayAction {
        action_id: "mcp:filesystem:tool:read_file".into(),
        upstream_id: "filesystem".into(),
        tool_name: "read_file".into(),
        title: "Read file".into(),
        description: Some("Read a file from disk".into()),
        risk: RiskLevel::ReadOnly,
        input_schema: serde_json::json!({"type": "object"}),
    }]);

    let docs = catalog.search_documents();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].action_id, "mcp:filesystem:tool:read_file");
    assert!(docs[0].text.contains("Read a file from disk"));
}
```

- [ ] **Step 3: Add search document export.**

Update `crates/agent-gateway/src/catalog.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewaySearchDocument {
    pub action_id: String,
    pub text: String,
}

impl ActionCatalog {
    pub fn search_documents(&self) -> Vec<GatewaySearchDocument> {
        self.actions
            .values()
            .map(|action| GatewaySearchDocument {
                action_id: action.action_id.clone(),
                text: format!(
                    "{} {} {}",
                    action.title,
                    action.description.clone().unwrap_or_default(),
                    action.tool_name
                ),
            })
            .collect()
    }
}
```

- [ ] **Step 4: Export search document type.**

Update `crates/agent-gateway/src/lib.rs`:

```rust
pub use catalog::GatewaySearchDocument;
```

- [ ] **Step 5: Verify search handoff sidecar tests.**

Run:

```bash
cargo test -p agent-gateway search_integration
```

Expected: PASS.

### Task 5: Verify Full Gateway Extraction

**Files:**
- Test sidecar: `crates/agent-gateway/src/*.rs` (`#[cfg(test)] mod tests`)
- Read: `docs/plans/extract-crates/agent-gateway.md`

- [ ] **Step 1: Run focused gateway tests.**

Run:

```bash
cargo test -p agent-gateway
```

Expected: PASS.

- [ ] **Step 2: Scan for Lab service leakage.**

Run:

```bash
rg -n "Plex|Sonarr|Radarr|Unraid|Gotify|LAB_|\\.lab|ServiceCatalog|VirtualServer" crates/agent-gateway
```

Expected: no output.

- [ ] **Step 3: Commit the gateway extraction slice.**

Run:

```bash
git add crates/agent-gateway docs/plans/extract-crates/agent-gateway.md
git commit -m "feat(gateway): extract launcher action catalog"
```

Expected: commit contains only `agent-gateway` implementation, tests, and this plan if executing this slice alone.
