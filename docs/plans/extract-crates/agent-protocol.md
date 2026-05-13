---
title: "agent-protocol Extraction Implementation Plan"
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
  - "docs/references/acp/docs/markdown/0028-agentclientprotocol-com-protocol-content.md"
  - "docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md"
  - "docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
  - "docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
  - "docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md"
  - "docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-protocol Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Define AgentCast-owned protocol-neutral models shared by MCP, ACP, registry, marketplace, gateway, API, and CLI crates.

**Architecture:** `agent-protocol` contains serializable models and conversion targets, not SDK clients or runtime behavior.

**Tech Stack:** Rust 2024, serde, serde_json, url, uuid, jiff.

---

## MVP Position

Protocol models are required before MCP catalog, gateway, CLI, and API work can converge.

## Lab Source Files

- `../lab/crates/lab-apis/src/acp.rs`
- `../lab/crates/lab-apis/src/acp/types.rs`
- `../lab/crates/lab-apis/src/mcpregistry.rs`
- `../lab/crates/lab-apis/src/acp_registry.rs`
- `../lab/crates/lab-apis/src/marketplace.rs`
- `../lab/crates/lab-apis/src/openacp.rs`
- `../lab/crates/lab/src/dispatch/gateway/types.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab protocol DTO source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`.
- MCP DTO, schema, tools, resources, prompts, and registry claims are cross-checked against `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`, `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`, `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`, `docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md`, `docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md`, and `docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md`.
- ACP DTO claims are cross-checked against `docs/references/acp/docs/markdown/0022-agentclientprotocol-com-protocol-overview.md`, `docs/references/acp/docs/markdown/0028-agentclientprotocol-com-protocol-content.md`, `docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md`, and `docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md`.

## Live Lab Findings

- `lab-apis/src/*` has useful serializable model families but also service-client code; only DTOs move to `agent-protocol`.
- `dispatch/gateway/types.rs` contains gateway DTOs that should be split between protocol facts and `agent-ui-contracts` view models.
- `mcpregistry.rs` and `acp_registry.rs` should become separate protocol modules so ACP registry remains post-v0.
- Marketplace and stash metadata should enter protocol only when more than one crate needs the serialized contract.

## Extraction Boundary

Extract:

- provider/tool/action metadata.
- launcher action and invocation result models.
- MCP registry package models.
- install-plan models shared with marketplace.
- ACP session/event models needed by `agent-acp`.

Leave behind:

- SDK clients.
- runtime handles.
- API response envelopes tied to Axum.
- CLI formatting details.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-protocol/src/lib.rs` - public exports for launcher, MCP, install, registry, and ACP models.
- Create: `crates/agent-protocol/src/launcher.rs` - launcher action and invocation DTOs.
- Create: `crates/agent-protocol/src/mcp.rs` - MCP catalog and upstream DTOs without RMCP types.
- Create: `crates/agent-protocol/src/registry.rs` - normalized registry DTOs shared by registry and UI/API.
- Create: `crates/agent-protocol/src/install.rs` - install-plan DTOs shared with marketplace/API/CLI.
- Create: `crates/agent-protocol/src/acp.rs` - post-v0 ACP protocol-neutral DTOs.
- Add source-side test sidecars for: `crates/agent-protocol/src/launcher.rs` - launcher serialization tests.
- Add source-side test sidecars for: `crates/agent-protocol/src/{registry,install}.rs` - registry DTO tests.
- Add source-side test sidecars for: `crates/agent-protocol/src/acp.rs` - ACP model tests required by `agent-acp`.

## Implementation Tasks

### Task 1: Add MCP Launcher Protocol Models

**Files:**
- Modify: `crates/agent-protocol/src/lib.rs`
- Create: `crates/agent-protocol/src/launcher.rs`
- Create: `crates/agent-protocol/src/mcp.rs`
- Test sidecar: `crates/agent-protocol/src/launcher.rs`

- [ ] **Step 1: Write failing serialization tests for launcher action IDs.**

Create a source-side test sidecar next to `crates/agent-protocol/src/launcher.rs` with:

```rust
use super::*;
use serde_json::json;

#[test]
fn launcher_action_serializes_stable_ids_and_schema() {
    let action = LauncherAction {
        action_id: "mcp:filesystem:tool:read_file".into(),
        upstream_id: "filesystem".into(),
        tool_name: "read_file".into(),
        title: "Read file".into(),
        description: Some("Read a local file".into()),
        input_schema: json!({"type": "object"}),
    };

    let value = serde_json::to_value(action).unwrap();
    assert_eq!(value["action_id"], "mcp:filesystem:tool:read_file");
    assert_eq!(value["upstream_id"], "filesystem");
}

#[test]
fn invocation_request_keeps_raw_params() {
    let request = InvocationRequest {
        action_id: "mcp:filesystem:tool:read_file".into(),
        params: json!({"path": "/tmp/a.txt"}),
    };

    assert_eq!(request.params["path"], "/tmp/a.txt");
}
```

Run:

```bash
cargo nextest run -p agent-protocol launcher
```

Expected: FAIL because launcher models do not exist yet.

- [ ] **Step 2: Export launcher and MCP modules.**

Update `crates/agent-protocol/src/lib.rs`:

```rust
mod launcher;
mod mcp;

pub use launcher::{InvocationRequest, InvocationResponse, LauncherAction};
pub use mcp::{McpCatalogSnapshot, McpUpstreamSummary};
```

- [ ] **Step 3: Implement launcher DTOs.**

Create `crates/agent-protocol/src/launcher.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LauncherAction {
    pub action_id: String,
    pub upstream_id: String,
    pub tool_name: String,
    pub title: String,
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvocationRequest {
    pub action_id: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvocationResponse {
    pub action_id: String,
    #[serde(default)]
    pub result: serde_json::Value,
}
```

- [ ] **Step 4: Implement MCP catalog DTOs.**

Create `crates/agent-protocol/src/mcp.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::LauncherAction;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpCatalogSnapshot {
    pub upstreams: Vec<McpUpstreamSummary>,
    pub actions: Vec<LauncherAction>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpUpstreamSummary {
    pub upstream_id: String,
    pub status: String,
    pub tool_count: usize,
}
```

- [ ] **Step 5: Verify launcher tests.**

Run:

```bash
cargo nextest run -p agent-protocol launcher
```

Expected: PASS.

### Task 2: Add Registry And Install DTOs

**Files:**
- Create: `crates/agent-protocol/src/registry.rs`
- Create: `crates/agent-protocol/src/install.rs`
- Modify: `crates/agent-protocol/src/lib.rs`
- Test sidecar: `crates/agent-protocol/src/{registry,install}.rs`

- [ ] **Step 1: Write failing registry/install DTO tests.**

Create a source-side test sidecar next to `crates/agent-protocol/src/{registry,install}.rs` with:

```rust
use super::*;

#[test]
fn registry_server_dto_round_trips() {
    let server = RegistryServerDto {
        name: "io.modelcontextprotocol/filesystem".into(),
        description: Some("Filesystem MCP server".into()),
        latest_version: Some("0.6.2".into()),
    };

    let value = serde_json::to_value(server).unwrap();
    assert_eq!(value["name"], "io.modelcontextprotocol/filesystem");
}

#[test]
fn install_plan_dto_keeps_preview_json() {
    let plan = InstallPlanDto {
        package: "io.modelcontextprotocol/filesystem".into(),
        steps: vec![InstallStepDto {
            kind: "add_mcp_upstream".into(),
            target: "mcp.upstreams.filesystem".into(),
            preview: serde_json::json!({"command": "npx"}),
        }],
    };

    assert_eq!(plan.steps[0].preview["command"], "npx");
}
```

- [ ] **Step 2: Export registry and install DTOs.**

Update `crates/agent-protocol/src/lib.rs`:

```rust
mod install;
mod registry;

pub use install::{InstallPlanDto, InstallStepDto};
pub use registry::RegistryServerDto;
```

- [ ] **Step 3: Implement DTOs.**

Create `crates/agent-protocol/src/registry.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryServerDto {
    pub name: String,
    pub description: Option<String>,
    pub latest_version: Option<String>,
}
```

Create `crates/agent-protocol/src/install.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallPlanDto {
    pub package: String,
    pub steps: Vec<InstallStepDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallStepDto {
    pub kind: String,
    pub target: String,
    pub preview: serde_json::Value,
}
```

- [ ] **Step 4: Verify registry/install tests.**

Run:

```bash
cargo nextest run -p agent-protocol registry
```

Expected: PASS.

### Task 3: Add ACP Models Needed By `agent-acp`

**Files:**
- Create: `crates/agent-protocol/src/acp.rs`
- Test sidecar: `crates/agent-protocol/src/acp.rs`

- [ ] **Step 1: Follow the ACP model task in `agent-acp.md`.**

Run:

```bash
sed -n '1,260p' docs/plans/extract-crates/agent-acp.md
```

Expected: ACP event, session, provider, permission, and content model requirements are available.

- [ ] **Step 2: Write ACP protocol tests from the ACP extraction plan.**

Create a source-side test sidecar next to `crates/agent-protocol/src/acp.rs` using the tests shown in `docs/plans/extract-crates/agent-acp.md` Task 2. The required assertions are:

```rust
use super::*;
use serde_json::json;

#[test]
fn acp_unknown_event_preserves_raw_payload() {
    let event = AcpEvent::Unknown {
        id: "evt-raw".into(),
        created_at: "2026-05-05T00:00:00Z".into(),
        session_id: "session-1".into(),
        seq: 7,
        event_kind: "provider_specific".into(),
        raw: json!({"nested": {"value": true}}),
    };

    let value = serde_json::to_value(&event).unwrap();
    assert_eq!(value["kind"], "unknown");
    assert_eq!(value["event_kind"], "provider_specific");
    assert_eq!(value["raw"]["nested"]["value"], true);
}
```

- [ ] **Step 3: Export and implement minimal ACP DTOs.**

Update `crates/agent-protocol/src/lib.rs`:

```rust
pub mod acp;
```

Create `crates/agent-protocol/src/acp.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AcpEvent {
    Unknown {
        id: String,
        created_at: String,
        session_id: String,
        seq: u64,
        event_kind: String,
        raw: serde_json::Value,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AcpContentBlock {
    Text { text: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcpPermissionOption {
    pub option_id: String,
    pub name: String,
    pub kind: String,
}
```

- [ ] **Step 4: Verify ACP DTO tests.**

Run:

```bash
cargo nextest run -p agent-protocol acp_types
```

Expected: PASS.

### Task 4: Separate Protocol DTOs From View DTOs

**Files:**
- Modify: `crates/agent-protocol/src/lib.rs`
- Read: `docs/plans/extract-crates/agent-ui-contracts.md`

- [ ] **Step 1: Classify each candidate type before adding it.**

Expected: reusable facts stay in `agent-protocol`; display-only shapes move to `agent-ui-contracts`.

- [ ] **Step 2: Scan protocol code for SDK/runtime/UI leakage.**

Run:

```bash
rg -n "rmcp|agent_client_protocol|axum|clap|tui|component|view_model|ViewModel" crates/agent-protocol
```

Expected: no output.

### Task 5: Verify Full Protocol Extraction

**Files:**
- Test sidecar: `crates/agent-protocol/src/*.rs`
- Read: `docs/plans/extract-crates/agent-protocol.md`

- [ ] **Step 1: Verify protocol crate.**

Run:

```bash
cargo nextest run -p agent-protocol
```

Expected: protocol tests pass.

- [ ] **Step 2: Commit the protocol extraction slice.**

Run:

```bash
git add crates/agent-protocol docs/plans/extract-crates/agent-protocol.md
git commit -m "feat(protocol): extract shared dtos"
```

Expected: commit contains only `agent-protocol` implementation, tests, and this plan if executing this slice alone.
