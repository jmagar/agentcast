---
title: "agent-server Extraction Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-server Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Compose AgentCast's binary entrypoint from config, CLI, runtime, gateway, API, registry, and marketplace crates.

**Architecture:** `agent-server` is the binary composition crate. It owns startup wiring, logging initialization, and top-level command dispatch; it must not own domain behavior.

**Tech Stack:** Rust 2024, Tokio, Clap, tracing-subscriber, AgentCast workspace crates.

---

## MVP Position

The server crate is needed once CLI command dispatch and optional HTTP serving are ready.

## Lab Source Files

- `../lab/crates/lab/src/main.rs`
- `../lab/crates/lab/src/cli.rs`
- `../lab/crates/lab/src/cli/serve.rs`
- `../lab/crates/lab/src/api/router.rs`
- `../lab/crates/lab/src/api/state.rs`
- `../lab/crates/lab/src/config.rs`
- `../lab/crates/lab/src/observability.rs`
- `../lab/crates/lab/src/log_fmt/formatter.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab server/binary composition source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`.
- MCP startup, lifecycle, transport, and tool exposure assumptions are cross-checked against `docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md`, `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`, and `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`.
- Claude Code MCP compatibility checks use `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`.

## Live Lab Findings

- `main.rs` shows binary composition, top-level command dispatch, config load, and tracing startup.
- `cli/serve.rs` and `api/router.rs` show bind, serve, route, CORS, and shutdown wiring.
- `observability.rs` and `log_fmt/formatter.rs` are useful startup logging references, but reusable setup belongs in `agent-observability`.
- Node/fleet startup and Lab service registry assembly should not move into `agent-server`.

## Extraction Boundary

Extract:

- startup logging and tracing setup.
- top-level CLI dispatch.
- config load before runtime creation.
- server serve/shutdown wiring.

Leave behind:

- Lab service registry assembly.
- homelab dotenv assumptions.
- node/fleet startup tasks.
- browser admin assets until AgentCast UI exists.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-server/src/main.rs` - binary entrypoint, CLI parsing, logging setup, and command dispatch.
- Create: `crates/agent-server/src/lib.rs` if the crate needs testable composition helpers.
- Create: `crates/agent-server/src/startup.rs` - startup composition helpers for config/logging/runtime handles.
- Create: `crates/agent-server/src/serve.rs` - optional HTTP bind/shutdown wiring.
- Add source-side test sidecars for: `crates/agent-server/src/startup.rs` - top-level CLI dispatch tests.
- Add source-side test sidecars for: `crates/agent-server/src/serve.rs` - HTTP serve wiring tests.
- Add source-side test sidecars for: `crates/agent-server/src/startup.rs` - thin-startup boundary tests.

## Implementation Tasks

### Task 1: Wire Top-Level CLI Dispatch

**Files:**
- Modify: `crates/agent-server/src/main.rs`
- Create: `crates/agent-server/src/lib.rs`
- Create: `crates/agent-server/src/startup.rs`
- Test sidecar: `crates/agent-server/src/startup.rs`

- [ ] **Step 1: Inspect Lab binary composition.**

Run:

```bash
sed -n '1,220p' ../lab/crates/lab/src/main.rs
sed -n '1,220p' ../lab/crates/lab/src/cli.rs
```

Expected: reusable startup and dispatch patterns are identified.

- [ ] **Step 2: Write failing CLI composition test.**

Create a source-side test sidecar next to `crates/agent-server/src/startup.rs` with:

```rust
use super::*;

#[tokio::test]
async fn dispatch_cli_parses_servers_list() {
    let outcome = dispatch_cli(["agentcast", "servers", "list"]).await.unwrap();
    assert_eq!(outcome, ServerCommandOutcome::CommandDispatched);
}
```

- [ ] **Step 3: Add testable server library surface.**

Create `crates/agent-server/src/lib.rs`:

```rust
mod startup;

pub use startup::{dispatch_cli, ServerCommandOutcome};
```

Create `crates/agent-server/src/startup.rs`:

```rust
use clap::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerCommandOutcome {
    PrintedAndExited,
    CommandDispatched,
}

pub async fn dispatch_cli<I, T>(args: I) -> anyhow::Result<ServerCommandOutcome>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let _cli = agent_cli::AgentCli::try_parse_from(args)?;
    Ok(ServerCommandOutcome::CommandDispatched)
}
```

- [ ] **Step 4: Wire `main.rs` through the library helper.**

Update `crates/agent-server/src/main.rs`:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    agent_observability::init_tracing();
    let _outcome = agent_server::dispatch_cli(std::env::args_os()).await?;
    Ok(())
}
```

- [ ] **Step 5: Verify CLI composition test.**

Run:

```bash
cargo nextest run -p agent-server server_cli
```

Expected: PASS once `agent-cli` exposes the version path expected by the test.

### Task 2: Add Optional HTTP Serve Wiring

**Files:**
- Modify: `crates/agent-server/src/main.rs`
- Create: `crates/agent-server/src/serve.rs`
- Modify: `crates/agent-server/src/lib.rs`
- Test sidecar: `crates/agent-server/src/serve.rs`

- [ ] **Step 1: Inspect Lab serve command.**

Run:

```bash
sed -n '1,260p' ../lab/crates/lab/src/cli/serve.rs
```

Expected: reusable bind/shutdown/logging patterns are identified.

- [ ] **Step 2: Write failing serve options test.**

Create a source-side test sidecar next to `crates/agent-server/src/serve.rs` with:

```rust
use super::*;

#[test]
fn serve_options_default_to_loopback() {
    let options = ServeOptions::default();
    assert_eq!(options.bind.to_string(), "127.0.0.1:0");
}
```

- [ ] **Step 3: Export serve types.**

Update `crates/agent-server/src/lib.rs`:

```rust
mod serve;
pub use serve::{ServeOptions, serve_http};
```

- [ ] **Step 4: Implement serve options and HTTP entrypoint.**

Create `crates/agent-server/src/serve.rs`:

```rust
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use agent_api::{ApiState, build_router};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServeOptions {
    pub bind: SocketAddr,
}

impl Default for ServeOptions {
    fn default() -> Self {
        Self {
            bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        }
    }
}

pub async fn serve_http(options: ServeOptions, state: ApiState) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(options.bind).await?;
    let app = build_router(state);
    axum::serve(listener, app).await?;
    Ok(())
}
```

- [ ] **Step 5: Verify serve tests.**

Run:

```bash
cargo nextest run -p agent-server server_serve
```

Expected: PASS.

### Task 3: Keep Startup Thin

**Files:**
- Modify: `crates/agent-server/src/main.rs`
- Test sidecar: `crates/agent-server/src/startup.rs`

- [ ] **Step 1: Verify composition stays in the binary crate.**

Run:

```bash
sed -n '1,220p' ../lab/crates/lab/src/main.rs
sed -n '1,180p' ../lab/crates/lab/src/observability.rs
```

Expected: `agent-server` wires crates together without owning config parsing, route definitions, CLI command definitions, or runtime policy.

- [ ] **Step 2: Write boundary test for startup module size.**

Create a source-side test sidecar next to `crates/agent-server/src/startup.rs` with:

```rust
#[test]
fn server_entrypoint_does_not_define_domain_models() {
    let main_rs = std::fs::read_to_string("src/main.rs").unwrap();
    assert!(!main_rs.contains("struct GatewayAction"));
    assert!(!main_rs.contains("struct McpUpstreamConfig"));
    assert!(!main_rs.contains("struct InstallPlan"));
}
```

- [ ] **Step 3: Verify composition tests.**

Run:

```bash
cargo nextest run -p agent-server server_composition
```

Expected: PASS.

### Task 4: Verify Full Server Extraction

**Files:**
- Test sidecar: `crates/agent-server/src/*.rs`
- Read: `docs/plans/extract-crates/agent-server.md`

- [ ] **Step 1: Run focused server tests.**

Run:

```bash
cargo nextest run -p agent-server
```

Expected: PASS.

- [ ] **Step 2: Scan for Lab service startup leakage.**

Run:

```bash
rg -n "Plex|Sonarr|Radarr|Unraid|Gotify|LAB_|\\.lab|NodeRuntime|ServiceRegistry" crates/agent-server
```

Expected: no output.

- [ ] **Step 3: Commit the server extraction slice.**

Run:

```bash
git add crates/agent-server docs/plans/extract-crates/agent-server.md
git commit -m "feat(server): extract binary composition"
```

Expected: commit contains only `agent-server` implementation, tests, and this plan if executing this slice alone.
