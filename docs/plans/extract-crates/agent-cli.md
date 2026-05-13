---
title: "agent-cli Extraction Implementation Plan"
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
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcporter/docs/cli-reference.md"
  - "docs/references/mcporter/docs/tool-calling.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-cli Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract Lab's typed CLI adapter pattern for the AgentCast MCP launcher MVP.

**Architecture:** `agent-cli` should own command parsing and human-facing output only. Commands call shared AgentCast runtime, gateway, registry, and marketplace APIs.

**Tech Stack:** Rust 2024, Clap, serde_json output, tracing initialization through server/main composition.

---

## MVP Position

The CLI is the first preferred surface for the MCP launcher MVP.

## Lab Source Files

- `../lab/crates/lab/src/cli.rs`
- `../lab/crates/lab/src/cli/helpers.rs`
- `../lab/crates/lab/src/cli/params.rs`
- `../lab/crates/lab/src/cli/gateway.rs`
- `../lab/crates/lab/src/cli/marketplace.rs`
- `../lab/crates/lab/src/cli/install.rs`
- `../lab/crates/lab/src/cli/serve.rs`
- `../lab/crates/lab/src/cli/health.rs`
- `../lab/crates/lab/src/cli/openacp.rs`
- `../lab/crates/lab/src/output/render.rs`
- `../lab/crates/lab/src/output/theme.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab CLI source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`.
- MCP server configuration and stdio/http/SSE naming claims are cross-checked against `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`, `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`, and `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`.
- CLI tool invocation ergonomics are compared with `docs/references/mcporter/docs/tool-calling.md` and `docs/references/mcporter/docs/cli-reference.md`; mcporter is a reference point, not a dependency.

## Live Lab Findings

- `cli.rs` and `cli/gateway.rs` show typed subcommand layout and dispatch delegation.
- `output/render.rs` and `output/theme.rs` are high-value for JSON/human output, color policy, ASCII fallback, and table rendering.
- `cli/marketplace.rs` and `cli/install.rs` are useful for preview/apply command shape, but Lab plugin targets must stay out of v0.
- Service-specific command modules are examples of what not to copy.

## Extraction Boundary

Extract:

- typed subcommand layout.
- JSON/table output helper patterns.
- confirmation prompts for destructive operations.
- command-to-runtime delegation.
- `serve` command composition shape.

Leave behind:

- homelab service subcommands.
- Lab deploy/node/fleet command policy.
- ACP typed commands until post-v0.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-cli/src/lib.rs` - public CLI parser and dispatch exports.
- Create: `crates/agent-cli/src/commands.rs` - Clap command tree for MVP surfaces.
- Create: `crates/agent-cli/src/error.rs` - CLI error type and exit code mapping.
- Create: `crates/agent-cli/src/output.rs` - JSON/human renderer and output mode selection.
- Create: `crates/agent-cli/src/mcp.rs` - `servers` and `actions` command handlers.
- Create: `crates/agent-cli/src/registry.rs` - registry search command handler.
- Create: `crates/agent-cli/src/marketplace.rs` - install-plan preview/apply command handler.
- Add source-side test sidecars for: `crates/agent-cli/src/commands.rs` - command parsing tests.
- Add source-side test sidecars for: `crates/agent-cli/src/output.rs` - JSON and human output tests.
- Add source-side test sidecars for: `crates/agent-cli/src/error.rs` - stable exit code tests.

## Implementation Tasks

### Task 1: Define MVP CLI Commands

**Files:**
- Modify: `crates/agent-cli/src/lib.rs`
- Create: `crates/agent-cli/src/commands.rs`
- Create: `crates/agent-cli/src/error.rs`
- Create: `crates/agent-cli/src/output.rs`
- Test sidecar: `crates/agent-cli/src/commands.rs`

- [ ] **Step 1: Read existing CLI docs.**

Run:

```bash
sed -n '1,220p' docs/MVP.md
sed -n '1,220p' docs/API.md
```

Expected: CLI commands map to MCP server config, launcher actions, invocation, registry search, and install-plan preview/apply.

- [ ] **Step 2: Write failing parse tests.**

Create a source-side test sidecar next to `crates/agent-cli/src/commands.rs` with:

```rust
use super::*;
use clap::Parser;

#[test]
fn parses_servers_list_json() {
    let cli = AgentCli::try_parse_from(["agentcast", "servers", "list", "--json"]).unwrap();
    assert_eq!(cli.output(), OutputMode::Json);
    assert!(matches!(
        cli.command,
        AgentCommand::Servers(ServersCommand::List)
    ));
}

#[test]
fn parses_action_call_with_json_params() {
    let cli = AgentCli::try_parse_from([
        "agentcast",
        "call",
        "mcp:filesystem:tool:read_file",
        "--params",
        r#"{"path":"/tmp/a.txt"}"#,
    ])
    .unwrap();

    let AgentCommand::Call { action_id, params } = cli.command else {
        panic!("expected call command");
    };
    assert_eq!(action_id, "mcp:filesystem:tool:read_file");
    assert_eq!(params, r#"{"path":"/tmp/a.txt"}"#);
}
```

Run:

```bash
cargo nextest run -p agent-cli cli_parse
```

Expected: FAIL because the Clap command types do not exist yet.

- [ ] **Step 3: Export CLI types.**

Update `crates/agent-cli/src/lib.rs`:

```rust
mod commands;
mod error;
mod output;

pub use commands::{
    ActionsCommand, AgentCli, AgentCommand, InstallCommand, RegistryCommand, ServersCommand,
};
pub use error::{CliError, CliResult, ExitCode};
pub use output::{OutputMode, render_json, render_table};
```

- [ ] **Step 4: Implement Clap command tree.**

Create `crates/agent-cli/src/commands.rs`:

```rust
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputMode {
    Human,
    Json,
}

#[derive(Debug, Parser)]
#[command(name = "agentcast")]
pub struct AgentCli {
    #[arg(long = "json", global = true, action = clap::ArgAction::SetTrue)]
    json: bool,
    #[command(subcommand)]
    pub command: AgentCommand,
}

impl AgentCli {
    pub fn output(&self) -> OutputMode {
        if self.json {
            OutputMode::Json
        } else {
            OutputMode::Human
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum AgentCommand {
    Servers(ServersCommand),
    Actions(ActionsCommand),
    Call {
        action_id: String,
        #[arg(long, default_value = "{}")]
        params: String,
    },
    Registry(RegistryCommand),
    Install(InstallCommand),
}

#[derive(Debug, Subcommand)]
pub enum ServersCommand {
    List,
    Add {
        id: String,
        command: String,
        #[arg(long = "arg")]
        args: Vec<String>,
    },
    Remove {
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ActionsCommand {
    List,
}

#[derive(Debug, Subcommand)]
pub enum RegistryCommand {
    Search {
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
}

#[derive(Debug, Subcommand)]
pub enum InstallCommand {
    Preview {
        package: String,
    },
    Apply {
        package: String,
        #[arg(long)]
        yes: bool,
    },
}
```

- [ ] **Step 5: Add CLI error and exit codes.**

Create `crates/agent-cli/src/error.rs`:

```rust
use thiserror::Error;

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    Usage = 2,
    Config = 20,
    Runtime = 30,
    Invocation = 40,
}

#[derive(Debug, Error)]
pub enum CliError {
    #[error("invalid command usage: {0}")]
    Usage(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("runtime error: {0}")]
    Runtime(String),
    #[error("invocation error: {0}")]
    Invocation(String),
}

impl CliError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Usage(_) => ExitCode::Usage,
            Self::Config(_) => ExitCode::Config,
            Self::Runtime(_) => ExitCode::Runtime,
            Self::Invocation(_) => ExitCode::Invocation,
        }
    }
}
```

- [ ] **Step 6: Verify parse tests.**

Run:

```bash
cargo nextest run -p agent-cli cli_parse
```

Expected: PASS.

### Task 2: Delegate CLI Commands To Shared Runtime

**Files:**
- Create: `crates/agent-cli/src/mcp.rs`
- Create: `crates/agent-cli/src/registry.rs`
- Create: `crates/agent-cli/src/marketplace.rs`
- Test sidecar: `crates/agent-cli/src/output.rs`

- [ ] **Step 1: Inspect Lab gateway CLI command shape.**

Run:

```bash
sed -n '1,260p' ../lab/crates/lab/src/cli/gateway.rs
```

Expected: reusable command grouping and output patterns are identified.

- [ ] **Step 2: Write failing command output tests.**

Create a source-side test sidecar next to `crates/agent-cli/src/output.rs` with:

```rust
use super::*;
use serde::Serialize;

#[derive(Serialize)]
struct Row {
    id: &'static str,
    status: &'static str,
}

#[test]
fn render_json_is_stable_and_machine_readable() {
    let output = render_json(&Row {
        id: "filesystem",
        status: "running",
    })
    .unwrap();

    assert_eq!(output, r#"{"id":"filesystem","status":"running"}"#);
}

#[test]
fn render_table_includes_headers_and_values() {
    let rows = [Row {
        id: "filesystem",
        status: "running",
    }];
    let output = render_table(["ID", "STATUS"], rows.iter().map(|row| [row.id, row.status]));

    assert!(output.contains("ID"));
    assert!(output.contains("filesystem"));
    assert!(output.contains("running"));
}
```

- [ ] **Step 3: Implement render helpers.**

Create `crates/agent-cli/src/output.rs`:

```rust
use serde::Serialize;

pub use crate::commands::OutputMode;

pub fn render_json(value: &impl Serialize) -> serde_json::Result<String> {
    serde_json::to_string(value)
}

pub fn render_table<const N: usize>(
    headers: [&str; N],
    rows: impl IntoIterator<Item = [&str; N]>,
) -> String {
    let mut output = String::new();
    output.push_str(&headers.join("  "));
    output.push('\n');
    for row in rows {
        output.push_str(&row.join("  "));
        output.push('\n');
    }
    output
}
```

- [ ] **Step 4: Add thin handler modules.**

Create `crates/agent-cli/src/mcp.rs`:

```rust
use crate::CliResult;

pub async fn list_servers() -> CliResult<serde_json::Value> {
    Ok(serde_json::json!({"servers": []}))
}

pub async fn list_actions() -> CliResult<serde_json::Value> {
    Ok(serde_json::json!({"actions": []}))
}
```

Create `crates/agent-cli/src/registry.rs`:

```rust
use crate::CliResult;

pub async fn search_registry(query: &str, limit: usize) -> CliResult<serde_json::Value> {
    Ok(serde_json::json!({"query": query, "limit": limit, "results": []}))
}
```

Create `crates/agent-cli/src/marketplace.rs`:

```rust
use crate::CliResult;

pub async fn preview_install(package: &str) -> CliResult<serde_json::Value> {
    Ok(serde_json::json!({"package": package, "changes": []}))
}
```

Expected: handlers compile as thin adapters and are later wired to `agent-runtime`, `agent-gateway`, `agent-registry`, and `agent-marketplace` without owning protocol behavior.

- [ ] **Step 5: Verify output tests.**

Run:

```bash
cargo nextest run -p agent-cli cli_output
```

Expected: PASS.

### Task 3: Port Output Contract Tests

**Files:**
- Create or modify: `crates/agent-cli/src/output.rs`
- Test sidecar: `crates/agent-cli/src/error.rs`

- [ ] **Step 1: Read Lab output behavior.**

Run:

```bash
rg -n "json_format|human_format|render_catalog|visible_width|symbol_mode|ascii" ../lab/crates/lab/src/output/render.rs ../lab/crates/lab/src/output/theme.rs
```

Expected: AgentCast CLI output supports stable JSON, human tables, non-TTY behavior, and ASCII fallback.

- [ ] **Step 2: Write failing exit-code tests.**

Create a source-side test sidecar next to `crates/agent-cli/src/error.rs` with:

```rust
use super::*;

#[test]
fn cli_errors_map_to_stable_exit_codes() {
    assert_eq!(CliError::Usage("bad flag".into()).exit_code(), ExitCode::Usage);
    assert_eq!(CliError::Config("missing config".into()).exit_code(), ExitCode::Config);
    assert_eq!(CliError::Runtime("server failed".into()).exit_code(), ExitCode::Runtime);
    assert_eq!(
        CliError::Invocation("tool failed".into()).exit_code(),
        ExitCode::Invocation
    );
}
```

- [ ] **Step 3: Verify exit-code tests.**

Run:

```bash
cargo nextest run -p agent-cli exit_codes
```

Expected: PASS.

### Task 4: Verify Full CLI Extraction

**Files:**
- Test sidecar: `crates/agent-cli/src/*.rs`
- Read: `docs/plans/extract-crates/agent-cli.md`

- [ ] **Step 1: Run focused CLI tests.**

Run:

```bash
cargo nextest run -p agent-cli
```

Expected: PASS.

- [ ] **Step 2: Scan for Lab service leakage.**

Run:

```bash
rg -n "Plex|Sonarr|Radarr|Unraid|Gotify|LAB_|\\.lab|deploy|node" crates/agent-cli
```

Expected: no output.

- [ ] **Step 3: Commit the CLI extraction slice.**

Run:

```bash
git add crates/agent-cli docs/plans/extract-crates/agent-cli.md
git commit -m "feat(cli): extract launcher command surface"
```

Expected: commit contains only `agent-cli` implementation, tests, and this plan if executing this slice alone.
