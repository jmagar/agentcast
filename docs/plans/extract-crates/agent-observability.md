---
title: "agent-observability Extraction Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md"
  - "docs/references/acp/docs/markdown/0062-agentclientprotocol-com-protocol-prompt-turn.md"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-observability Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `agent-observability` as the reusable tracing, audit, redaction, health, and metrics helper crate for AgentCast.

**Architecture:** Observability must support runtime, gateway, API, CLI, and desktop/web surfaces without owning product behavior. Extract generic Lab logging and diagnostics patterns only when they are independent of Lab service administration.

**Tech Stack:** Rust 2024, tracing, tracing-subscriber, serde/serde_json, thiserror, jiff, uuid.

---

## MVP Position

`agent-observability` can support v0 when it keeps logging and audit behavior small. It must not block MCP launcher work with dashboards, metrics exporters, or long-term telemetry storage.

## Lab Evidence Read

- `../lab/crates/lab/src/observability.rs`
- `../lab/crates/lab/src/observability/activity.rs`
- `../lab/crates/lab/src/observability/activity_event.rs`
- `../lab/crates/lab/src/audit.rs`
- `../lab/crates/lab/src/audit/types.rs`
- `../lab/crates/lab/src/log_fmt/formatter.rs`
- `../lab/crates/lab/src/dispatch/logs/store.rs`
- `../lab/crates/lab/src/dispatch/logs/stream.rs`
- `../lab/crates/lab/src/dispatch/logs/types.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab observability source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`.
- Protocol event and notification assumptions are cross-checked against MCP lifecycle/tool notifications in `docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md` and `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`, plus ACP prompt/tool update semantics in `docs/references/acp/docs/markdown/0062-agentclientprotocol-com-protocol-prompt-turn.md` and `docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md`.

Live source discovery command:

```bash
rg -n "tracing|tracing_subscriber|audit|redact|health|metrics|span" ../lab crates docs
```

## Live Lab Findings

- Lab separates activity events from lower-level log formatting; AgentCast should do the same.
- Log ingestion and node/peer streaming are useful post-v0 evidence but should not block v0 launcher logs.
- Redaction behavior appears across config, gateway projection, setup secret masks, and logs; `agent-observability` should provide reusable helpers once the core secret type is settled.

## Extraction Boundary

Extract into `agent-observability`:

- tracing initialization helpers that are not binary-specific.
- redaction helpers for secrets and environment-like values.
- audit event structs for launcher actions, install plans, server lifecycle, and invocation results.
- health snapshot DTOs that can be reused by CLI/API/UI surfaces.
- span naming conventions and operation metadata helpers.

Keep out of `agent-observability`:

- process lifecycle ownership.
- SQLite or file persistence.
- API route handlers.
- CLI rendering.
- Lab service names, service health policy, and homelab-specific dashboards.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-observability/src/lib.rs` - public exports for tracing, redaction, activity, audit, and health helpers.
- Create: `crates/agent-observability/src/tracing.rs` - reusable tracing initialization.
- Create: `crates/agent-observability/src/redaction.rs` - secret-like key/value redaction helpers.
- Create: `crates/agent-observability/src/activity.rs` - structured activity event DTOs.
- Create: `crates/agent-observability/src/health.rs` - health snapshot DTOs.
- Add source-side test sidecars for: `crates/agent-observability/src/redaction.rs` - redaction tests.
- Add source-side test sidecars for: `crates/agent-observability/src/activity.rs` - activity event serialization tests.
- Add source-side test sidecars for: `crates/agent-observability/src/health.rs` - health DTO tests.

## Implementation Tasks

### Task 1: Split Activity Events From Log Formatting

**Files:**
- Create: `crates/agent-observability/src/activity.rs`
- Create: `crates/agent-observability/src/redaction.rs`
- Test sidecar: `crates/agent-observability/src/activity.rs`

- [ ] **Step 1: Read Lab activity and formatter sources.**

Run:

```bash
sed -n '1,220p' ../lab/crates/lab/src/observability/activity_event.rs
sed -n '1,180p' ../lab/crates/lab/src/log_fmt/formatter.rs
```

Expected: AgentCast keeps event DTOs independent from terminal/log formatting.

- [ ] **Step 2: Write failing activity event tests.**

Create a source-side test sidecar next to `crates/agent-observability/src/activity.rs` with:

```rust
use super::*;

#[test]
fn activity_event_serializes_stable_kind() {
    let event = ActivityEvent {
        id: "evt-1".into(),
        kind: ActivityKind::InvocationStarted,
        target: "mcp:filesystem:tool:read_file".into(),
        message: "started".into(),
    };

    let value = serde_json::to_value(event).unwrap();
    assert_eq!(value["kind"], "invocation_started");
    assert_eq!(value["target"], "mcp:filesystem:tool:read_file");
}
```

- [ ] **Step 3: Export activity types.**

Update `crates/agent-observability/src/lib.rs`:

```rust
mod activity;
mod health;
mod redaction;
mod tracing;

pub use activity::{ActivityEvent, ActivityKind};
pub use health::{HealthSnapshot, HealthStatus};
pub use redaction::{redact_env, redact_value};
pub use tracing::init_tracing;
```

- [ ] **Step 4: Implement activity DTOs.**

Create `crates/agent-observability/src/activity.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub id: String,
    pub kind: ActivityKind,
    pub target: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityKind {
    InvocationStarted,
    InvocationCompleted,
    InstallPlanPreviewed,
    ServerStarted,
    ServerStopped,
}
```

### Task 2: Add Redaction Helpers

**Files:**
- Create: `crates/agent-observability/src/redaction.rs`
- Test sidecar: `crates/agent-observability/src/redaction.rs`

- [ ] **Step 1: Write failing redaction tests.**

Create a source-side test sidecar next to `crates/agent-observability/src/redaction.rs` with:

```rust
use super::*;

#[test]
fn redacts_secret_like_env_keys() {
    let redacted = redact_env([("API_TOKEN", "secret"), ("SAFE_FLAG", "true")]);
    assert_eq!(redacted.get("API_TOKEN").map(String::as_str), Some("[REDACTED]"));
    assert_eq!(redacted.get("SAFE_FLAG").map(String::as_str), Some("true"));
}

#[test]
fn redact_value_preserves_short_non_secret_values() {
    assert_eq!(redact_value("hello"), "hello");
}
```

- [ ] **Step 2: Implement redaction.**

Create `crates/agent-observability/src/redaction.rs`:

```rust
use std::collections::BTreeMap;

pub fn redact_value(value: &str) -> String {
    value.to_string()
}

pub fn redact_env<'a>(
    entries: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> BTreeMap<String, String> {
    entries
        .into_iter()
        .map(|(key, value)| {
            let lower = key.to_ascii_lowercase();
            let redacted = lower.contains("token")
                || lower.contains("secret")
                || lower.contains("password")
                || lower.contains("key");
            (
                key.to_string(),
                if redacted {
                    "[REDACTED]".into()
                } else {
                    value.to_string()
                },
            )
        })
        .collect()
}
```

- [ ] **Step 3: Verify redaction tests.**

Run:

```bash
cargo nextest run -p agent-observability redaction
```

Expected: PASS.

### Task 3: Add Health Snapshots And Tracing Setup

**Files:**
- Create: `crates/agent-observability/src/health.rs`
- Create: `crates/agent-observability/src/tracing.rs`
- Test sidecar: `crates/agent-observability/src/health.rs`

- [ ] **Step 1: Write failing health DTO tests.**

Create a source-side test sidecar next to `crates/agent-observability/src/health.rs` with:

```rust
use super::*;

#[test]
fn health_snapshot_serializes_status() {
    let snapshot = HealthSnapshot {
        component: "runtime".into(),
        status: HealthStatus::Healthy,
        message: None,
    };

    let value = serde_json::to_value(snapshot).unwrap();
    assert_eq!(value["status"], "healthy");
}
```

- [ ] **Step 2: Implement health DTOs and tracing init.**

Create `crates/agent-observability/src/health.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthSnapshot {
    pub component: String,
    pub status: HealthStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}
```

Create `crates/agent-observability/src/tracing.rs`:

```rust
pub fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}
```

- [ ] **Step 3: Verify health tests.**

Run:

```bash
cargo nextest run -p agent-observability health
```

Expected: PASS.

### Task 4: Verify Full Observability Extraction

**Files:**
- Test sidecar: `crates/agent-observability/src/*.rs`
- Read: `docs/plans/extract-crates/agent-observability.md`

- [ ] **Step 1: Run focused observability tests.**

Run:

```bash
cargo nextest run -p agent-observability
```

Expected: all redaction, audit DTO, and health DTO tests pass without launching external services.

- [ ] **Step 2: Scan for Lab service leakage.**

Run:

```bash
rg -n "Plex|Sonarr|Radarr|Unraid|Gotify|LAB_|\\.lab|node|fleet" crates/agent-observability
```

Expected: no output.
