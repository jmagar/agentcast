---
title: "agent-ui-contracts Extraction Implementation Plan"
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
  - "docs/references/acp/docs/markdown/0062-agentclientprotocol-com-protocol-prompt-turn.md"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
  - "docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-ui-contracts Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `agent-ui-contracts` as the shared client-facing DTO crate for web, desktop, Raycast-style, and external clients.

**Architecture:** The seed docs call this idea `agent-ui-contracts`; in AgentCast it should use the crate naming convention `agent-ui-contracts`. It must reuse `agent-protocol` and `agent-schema` models where possible and only add view/client shapes when the raw protocol model is not enough.

**Tech Stack:** Rust 2024, serde/serde_json, thiserror, uuid, jiff, url.

---

## MVP Position

`agent-ui-contracts` can remain skeletal until the API/web/desktop surfaces need shared view models. For v0, prefer minimal DTOs for launcher action lists, server status, invocation results, and install-plan preview if these appear in both API and UI.

## Lab Evidence Read

- `../lab/crates/lab/src/dispatch/gateway/types.rs`
- `../lab/crates/lab/src/dispatch/gateway/view_models.rs`
- `../lab/crates/lab/src/api/services/gateway.rs`
- `../lab/crates/lab/src/api/services/marketplace.rs`
- `../lab/crates/lab/src/api/services/registry_v01.rs`
- `../lab/apps/gateway-admin/lib/api/gateway-client.ts`
- `../lab/apps/gateway-admin/lib/api/marketplace-client.ts`
- `../lab/apps/gateway-admin/lib/acp/types.ts`
- `../lab/apps/gateway-admin/components/chat/types.ts`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab UI contract source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`.
- MCP view-model assumptions for tools, resources, prompts, and content are cross-checked against `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`, `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`, `docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md`, and `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`.
- ACP chat/content/tool-call view assumptions are cross-checked against `docs/references/acp/docs/markdown/0062-agentclientprotocol-com-protocol-prompt-turn.md`, `docs/references/acp/docs/markdown/0028-agentclientprotocol-com-protocol-content.md`, and `docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md`.

Live source discovery command:

```bash
rg -n "View|Dto|Response|Request|Bridge|Admin|Gateway|launcher|install plan|server status" ../lab/crates ../lab/apps ../lab/packages 2>/dev/null || true
```

## Live Lab Findings

- Rust gateway types and TS API clients already form an implicit DTO contract in Lab.
- AgentCast should define shared DTOs in Rust where API, desktop, web, and external clients need the same JSON shape.
- React components, Lab copy, and `Bridge*` compatibility names should stay out of this crate.

## Extraction Boundary

Extract into `agent-ui-contracts`:

- shared launcher list/detail view DTOs.
- server health/status view DTOs.
- install-plan preview DTOs.
- invocation result/error view DTOs.
- schema form metadata DTOs derived from `agent-schema`.
- versioned API/client envelope shapes when they are stable enough to share.

Keep out of `agent-ui-contracts`:

- Axum route handlers.
- React/Tauri components.
- CLI rendering.
- MCP/ACP SDK types.
- Lab `Bridge*` compatibility names unless used only as historical evidence.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-ui-contracts/src/lib.rs` - public exports for gateway, marketplace, registry, and invocation view DTOs.
- Create: `crates/agent-ui-contracts/src/gateway.rs` - launcher action and server status view DTOs.
- Create: `crates/agent-ui-contracts/src/marketplace.rs` - install preview view DTOs.
- Create: `crates/agent-ui-contracts/src/registry.rs` - registry result view DTOs.
- Create: `crates/agent-ui-contracts/src/invocation.rs` - invocation result/error view DTOs.
- Add source-side test sidecars for: `crates/agent-ui-contracts/src/{gateway,marketplace,registry,invocation}.rs` - JSON sidecar contract tests.
- Add source-side test sidecars for: `crates/agent-ui-contracts/src/lib.rs` - protocol reuse boundary tests.

## Implementation Tasks

### Task 1: Extract View DTOs From Protocol DTOs

**Files:**
- Create: `crates/agent-ui-contracts/src/gateway.rs`
- Create: `crates/agent-ui-contracts/src/marketplace.rs`
- Test sidecar: `crates/agent-ui-contracts/src/{gateway,marketplace,registry,invocation}.rs`

- [ ] **Step 1: Compare Rust route DTOs and TS client expectations.**

Run:

```bash
rg -n "Gateway.*View|ServerView|Marketplace|Registry|type .*Response|interface .*Response" ../lab/crates/lab/src/dispatch/gateway/types.rs ../lab/apps/gateway-admin/lib/api ../lab/apps/gateway-admin/lib/acp
```

Expected: shared JSON DTOs are stable, display-focused, and do not duplicate protocol models unless the view shape truly differs.

- [ ] **Step 2: Write failing serialization tests.**

Create a source-side test sidecar next to `crates/agent-ui-contracts/src/{gateway,marketplace,registry,invocation}.rs` with:

```rust
use super::*;

#[test]
fn action_list_item_view_serializes_client_shape() {
    let view = ActionListItemView {
        action_id: "mcp:filesystem:tool:read_file".into(),
        title: "Read file".into(),
        subtitle: Some("filesystem".into()),
        risk: "read_only".into(),
    };

    let value = serde_json::to_value(view).unwrap();
    assert_eq!(value["action_id"], "mcp:filesystem:tool:read_file");
    assert_eq!(value["subtitle"], "filesystem");
}

#[test]
fn server_status_view_keeps_display_status() {
    let view = ServerStatusView {
        upstream_id: "filesystem".into(),
        display_name: "Filesystem".into(),
        status: "running".into(),
    };

    let value = serde_json::to_value(view).unwrap();
    assert_eq!(value["status"], "running");
}
```

- [ ] **Step 3: Export UI contract modules.**

Update `crates/agent-ui-contracts/src/lib.rs`:

```rust
mod gateway;
mod invocation;
mod marketplace;
mod registry;

pub use gateway::{ActionListItemView, ServerStatusView};
pub use invocation::{InvocationErrorView, InvocationResultView};
pub use marketplace::InstallPreviewView;
pub use registry::RegistryResultView;
```

- [ ] **Step 4: Implement gateway view DTOs.**

Create `crates/agent-ui-contracts/src/gateway.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionListItemView {
    pub action_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub risk: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerStatusView {
    pub upstream_id: String,
    pub display_name: String,
    pub status: String,
}
```

- [ ] **Step 5: Implement marketplace, registry, and invocation view DTOs.**

Create `crates/agent-ui-contracts/src/marketplace.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallPreviewView {
    pub package: String,
    pub summary: String,
    pub changes: Vec<serde_json::Value>,
}
```

Create `crates/agent-ui-contracts/src/registry.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryResultView {
    pub name: String,
    pub description: Option<String>,
    pub latest_version: Option<String>,
}
```

Create `crates/agent-ui-contracts/src/invocation.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvocationResultView {
    pub action_id: String,
    pub output: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvocationErrorView {
    pub action_id: String,
    pub kind: String,
    pub message: String,
}
```

### Task 2: Prefer Protocol Reuse Over Duplication

**Files:**
- Create: `crates/agent-ui-contracts/src/lib.rs`
- Read: `crates/agent-protocol/src/lib.rs`

- [ ] **Step 1: Write protocol reuse boundary test.**

Create a source-side test sidecar next to `crates/agent-ui-contracts/src/lib.rs` with:

```rust
#[test]
fn ui_contracts_do_not_define_protocol_runtime_clients() {
    let lib = std::fs::read_to_string("src/lib.rs").unwrap();
    assert!(!lib.contains("McpClient"));
    assert!(!lib.contains("AcpProvider"));
    assert!(!lib.contains("GatewayRouter"));
}
```

- [ ] **Step 2: Verify protocol reuse test.**

Run:

```bash
cargo nextest run -p agent-ui-contracts protocol_reuse
```

Expected: PASS.

### Task 3: Verify Full UI Contracts Extraction

**Files:**
- Test sidecar: `crates/agent-ui-contracts/src/*.rs`
- Read: `docs/plans/extract-crates/agent-ui-contracts.md`

- [ ] **Step 1: Run focused UI contract tests.**

Run:

```bash
cargo nextest run -p agent-ui-contracts
```

Expected: DTO serialization tests pass without frontend build tooling.

- [ ] **Step 2: Scan for React/Tauri/rendering leakage.**

Run:

```bash
rg -n "react|tsx|component|tauri|clap|axum|rmcp|agent_client_protocol|Bridge" crates/agent-ui-contracts
```

Expected: no output.
