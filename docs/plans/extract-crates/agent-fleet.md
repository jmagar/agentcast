---
title: "agent-fleet Extraction Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
related: []
last_reviewed: "2026-05-15"
last_modified: "2026-05-15"
modified_on_branch: "gateway-first-skeleton"
modified_at_version: "0.1.0"
modified_at_commit: "d327495"
review_basis: "cross-referenced against gateway-first implementation audit and local docs/references snapshot"
---

# agent-fleet Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `agent-fleet` as the future crate for remote runtime/node coordination.

**Architecture:** Fleet behavior is post-v0 and must not leak Lab node or homelab assumptions into AgentCast. The crate should eventually define generic node identity, heartbeat, capability, and remote execution coordination contracts.

**Tech Stack:** Rust 2024, tokio, serde/serde_json, thiserror, tracing, jiff, uuid, url.

---

## MVP Position

`agent-fleet` is post-v0 unless explicitly promoted in `docs/MVP.md`. The MCP launcher MVP is local-first and should not depend on fleet behavior.

## Current Implementation Audit

As of 2026-05-15 on `gateway-first-skeleton`, `agent-fleet` implements the post-v0 contract layer called for by this plan: generic node identity, labels, heartbeat/status DTOs, capability summaries, execution targets, and remote execution request DTOs. Remote node enrollment, queueing, API/CLI routes, host management, and Lab fleet policy remain outside the local gateway.

## Lab Evidence Read

- `../lab/crates/lab/src/node/identity.rs`
- `../lab/crates/lab/src/node/enrollment.rs`
- `../lab/crates/lab/src/node/runtime.rs`
- `../lab/crates/lab/src/node/checkin.rs`
- `../lab/crates/lab/src/node/queue.rs`
- `../lab/crates/lab/src/node/store.rs`
- `../lab/crates/lab/src/node/health.rs`
- `../lab/crates/lab/src/api/nodes/fleet.rs`
- `../lab/crates/lab/src/api/nodes/status.rs`
- `../lab/crates/lab/src/dispatch/node/send.rs`
- `../lab/crates/lab/tests/node_queue.rs`
- `../lab/crates/lab/tests/node_identity.rs`
- `../lab/crates/lab/tests/device_runtime.rs`
- `../lab/crates/lab/tests/device_identity.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab fleet source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`; wildcard source paths must be expanded to explicit files before implementation.
- Fleet is post-v0 and has no direct upstream ACP/MCP protocol ownership. The only protocol-adjacent claim here, that local MCP stdio lifecycle belongs outside fleet, is cross-checked against `docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md` and `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`.

Live source discovery command:

```bash
rg -n "fleet|node|heartbeat|remote|capabilit|worker|agent host" ../lab
```

## Live Lab Findings

- Lab has a substantial node/fleet system: identity, enrollment, check-in, queue, runtime, log collection, API routes, and tests.
- AgentCast should extract only generic remote runtime contracts and leave Lab master/worker policy behind.
- Fleet remains post-v0 unless `docs/MVP.md` explicitly promotes remote execution.

## Extraction Boundary

Extract into `agent-fleet`:

- generic remote runtime/node identity models.
- heartbeat and capability summary DTOs.
- remote execution planning interfaces when they are independent of transport.
- health classification that does not assume Lab infrastructure.

Keep out of `agent-fleet`:

- local MCP stdio process lifecycle.
- launcher action routing for local tools.
- Lab-specific node policy, service inventory, and host management.
- API routes and CLI commands.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-fleet/src/lib.rs` - public exports for node identity, heartbeat, capability, and remote execution contracts.
- Create: `crates/agent-fleet/src/node.rs` - generic node identity and labels.
- Create: `crates/agent-fleet/src/heartbeat.rs` - heartbeat DTOs and status.
- Create: `crates/agent-fleet/src/capability.rs` - remote capability summary.
- Create: `crates/agent-fleet/src/execution.rs` - post-v0 remote execution request DTOs.
- Add source-side test sidecars for: `crates/agent-fleet/src/{node,heartbeat,capability}.rs` - serialization contract tests.
- Add source-side test sidecars for: `crates/agent-fleet/src/lib.rs` - local MVP isolation tests.

## Implementation Tasks

### Task 1: Freeze Generic Fleet Contracts Before Runtime

**Files:**
- Create: `crates/agent-fleet/src/node.rs`
- Create: `crates/agent-fleet/src/heartbeat.rs`
- Test sidecar: `crates/agent-fleet/src/{node,heartbeat,capability}.rs`

- [ ] **Step 1: Read Lab node identity and runtime evidence.**

Run:

```bash
rg -n "pub struct|pub enum|fn .*heartbeat|identity|enrollment|queue" ../lab/crates/lab/src/node ../lab/crates/lab/src/api/nodes
```

Expected: AgentCast starts with serializable contracts only and does not implement Lab node control policy.

- [ ] **Step 2: Write failing fleet contract tests.**

Create a source-side test sidecar next to `crates/agent-fleet/src/{node,heartbeat,capability}.rs` with:

```rust
use super::*;

#[test]
fn fleet_node_serializes_generic_identity() {
    let node = FleetNode {
        node_id: "node-1".into(),
        display_name: "Workshop".into(),
        labels: vec!["linux".into()],
    };

    let value = serde_json::to_value(node).unwrap();
    assert_eq!(value["node_id"], "node-1");
    assert_eq!(value["labels"][0], "linux");
}

#[test]
fn heartbeat_reports_capabilities_without_lab_policy() {
    let heartbeat = FleetHeartbeat {
        node_id: "node-1".into(),
        status: FleetStatus::Online,
        capabilities: vec![FleetCapability {
            name: "mcp_stdio".into(),
            version: Some("1".into()),
        }],
    };

    let value = serde_json::to_value(heartbeat).unwrap();
    assert_eq!(value["status"], "online");
}
```

- [ ] **Step 3: Export fleet modules.**

Update `crates/agent-fleet/src/lib.rs`:

```rust
mod capability;
mod heartbeat;
mod node;

pub use capability::FleetCapability;
pub use heartbeat::{FleetHeartbeat, FleetStatus};
pub use node::FleetNode;
```

- [ ] **Step 4: Implement generic fleet DTOs.**

Create `crates/agent-fleet/src/node.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetNode {
    pub node_id: String,
    pub display_name: String,
    pub labels: Vec<String>,
}
```

Create `crates/agent-fleet/src/capability.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetCapability {
    pub name: String,
    pub version: Option<String>,
}
```

Create `crates/agent-fleet/src/heartbeat.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::FleetCapability;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetHeartbeat {
    pub node_id: String,
    pub status: FleetStatus,
    pub capabilities: Vec<FleetCapability>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FleetStatus {
    Online,
    Degraded,
    Offline,
}
```

### Task 2: Preserve Post-v0 Boundary

**Files:**
- Create: `crates/agent-fleet/src/lib.rs`
- Read: `docs/MVP.md`

- [ ] **Step 1: Write boundary test.**

Create a source-side test sidecar next to `crates/agent-fleet/src/lib.rs` with:

```rust
#[test]
fn fleet_crate_does_not_define_local_mcp_runtime_types() {
    let lib = std::fs::read_to_string("src/lib.rs").unwrap();
    assert!(!lib.contains("ProcessHandle"));
    assert!(!lib.contains("McpClient"));
    assert!(!lib.contains("GatewayRouter"));
}
```

- [ ] **Step 2: Verify fleet tests.**

Run:

```bash
cargo nextest run -p agent-fleet
```

Expected: PASS.

### Task 3: Verify Full Fleet Extraction

**Files:**
- Test sidecar: `crates/agent-fleet/src/*.rs`
- Read: `docs/plans/extract-crates/agent-fleet.md`

- [ ] **Step 1: Confirm v0 crates do not depend on fleet.**

Run:

```bash
rg -n "agent-fleet|agent_fleet" crates/agent-config crates/agent-mcp crates/agent-runtime crates/agent-gateway crates/agent-cli crates/agent-server
```

Expected: no output until fleet is promoted beyond post-v0.

- [ ] **Step 2: Run focused fleet tests.**

Run:

```bash
cargo nextest run -p agent-fleet
```

Expected: model and contract tests pass without remote hosts.
