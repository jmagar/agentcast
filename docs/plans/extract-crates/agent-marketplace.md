---
title: "agent-marketplace Extraction Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/claude-code/markdown/0093-code-claude-com-docs-en-plugins.md"
  - "docs/references/claude-code/markdown/0102-code-claude-com-docs-en-plugins-reference.md"
  - "docs/references/claude-code/markdown/0108-code-claude-com-docs-en-plugin-marketplaces.md"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md"
  - "docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-marketplace Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract Lab's install-plan and marketplace preview/apply patterns for AgentCast packages, starting with MCP servers.

**Architecture:** `agent-marketplace` converts registry metadata into deterministic install plans. Registry lookup stays in `agent-registry`; config writes go through `agent-config`; runtime validation stays outside marketplace.

**Tech Stack:** Rust 2024, serde, serde_json, url, AgentCast registry/protocol models.

---

## MVP Position

For v0, marketplace is limited to official MCP Registry results and previewable MCP server install plans.

## Lab Source Files

- `../lab/crates/lab/src/dispatch/marketplace/catalog.rs`
- `../lab/crates/lab/src/dispatch/marketplace/client.rs`
- `../lab/crates/lab/src/dispatch/marketplace/diff.rs`
- `../lab/crates/lab/src/dispatch/marketplace/dispatch.rs`
- `../lab/crates/lab/src/dispatch/marketplace/fork.rs`
- `../lab/crates/lab/src/dispatch/marketplace/acp_catalog.rs`
- `../lab/crates/lab/src/dispatch/marketplace/acp_client.rs`
- `../lab/crates/lab/src/dispatch/marketplace/acp_dispatch.rs`
- `../lab/crates/lab/src/api/services/marketplace.rs`
- `../lab/crates/lab/src/cli/marketplace.rs`
- `../lab/crates/lab/src/dispatch/marketplace/mcp_catalog.rs`
- `../lab/crates/lab/src/dispatch/marketplace/mcp_dispatch.rs`
- `../lab/crates/lab/src/dispatch/marketplace/mcp_params.rs`
- `../lab/crates/lab/src/dispatch/marketplace/package.rs`
- `../lab/crates/lab/src/dispatch/marketplace/store.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab marketplace source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`.
- MCP Registry and package/install claims are cross-checked against `docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md` and `docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md`; the registry is preview, so marketplace plans must preserve fallback/error handling.
- Claude Code plugin marketplace claims are post-v0 and are cross-checked against `docs/references/claude-code/markdown/0093-code-claude-com-docs-en-plugins.md`, `docs/references/claude-code/markdown/0102-code-claude-com-docs-en-plugins-reference.md`, and `docs/references/claude-code/markdown/0108-code-claude-com-docs-en-plugin-marketplaces.md`.

## Live Lab Findings

- `mcp_dispatch.rs` contains the most relevant v0 MCP install flow: registry lookup, target parsing, stdio/http install branches, env resolution, and install observability. AgentCast must narrow env handling so durable non-secret settings become `config.toml` mutations and `.env` receives only secrets, endpoint URLs, tokens, API keys, or runtime process env values.
- `mcp_params.rs` has important safety rules: runtime allowlist, dangerous argv rejection, protected env names, env value validation, and registry URL validation.
- `store.rs` is registry cache/storage, so its persistence patterns belong in `agent-store` or `agent-registry`, not marketplace core.
- `package.rs` and non-MCP backend files are post-v0 plugin marketplace evidence.

## Extraction Boundary

Extract:

- install-plan preview/apply vocabulary.
- conflict detection.
- package diff and metadata normalization patterns.
- dry-run output patterns.
- config mutation plans that separate `config.toml` changes from `.env` value writes.

Leave behind:

- Lab plugin install targets.
- ACP Registry installs until post-v0.
- homelab service marketplace entries.
- direct UI card behavior.
- generated `.env` files containing package metadata, schemas, descriptions, defaults, or install history.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-marketplace/src/lib.rs` - public exports for install plans, conflicts, MCP planning, safety checks, and errors.
- Create: `crates/agent-marketplace/src/error.rs` - marketplace error type and stable kind strings.
- Create: `crates/agent-marketplace/src/plan.rs` - install plan, steps, preview/apply vocabulary.
- Create: `crates/agent-marketplace/src/conflict.rs` - conflict detection models.
- Create: `crates/agent-marketplace/src/mcp.rs` - MCP Registry package to AgentCast config mutation planner.
- Create: `crates/agent-marketplace/src/mcp_params.rs` - MCP install safety validation.
- Add sidecar tests in: `crates/agent-marketplace/src/plan.rs` (`#[cfg(test)] mod tests`) - plan and conflict tests.
- Add sidecar tests in: `crates/agent-marketplace/src/mcp.rs` (`#[cfg(test)] mod tests`) - MCP registry to plan tests.
- Add sidecar tests in: `crates/agent-marketplace/src/mcp_params.rs` (`#[cfg(test)] mod tests`) - unsafe runtime/env/URL rejection tests.

## Implementation Tasks

### Task 1: Define MCP Install Plan Model

**Files:**
- Modify: `crates/agent-marketplace/src/lib.rs`
- Create: `crates/agent-marketplace/src/error.rs`
- Create: `crates/agent-marketplace/src/plan.rs`
- Create: `crates/agent-marketplace/src/conflict.rs`
- Test sidecar: `crates/agent-marketplace/src/plan.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Inspect Lab marketplace diff behavior.**

Run:

```bash
sed -n '1,260p' ../lab/crates/lab/src/dispatch/marketplace/diff.rs
```

Expected: reusable preview and conflict patterns are identified.

- [ ] **Step 2: Write failing install-plan tests.**

Add this `#[cfg(test)] mod tests` sidecar to `crates/agent-marketplace/src/plan.rs`:

```rust
use super::*;

#[test]
fn install_plan_preview_is_deterministic() {
    let plan = InstallPlan::new("io.modelcontextprotocol/filesystem")
        .step(InstallStep {
            kind: InstallStepKind::AddMcpUpstream,
            description: "Add filesystem MCP upstream".into(),
            target: "mcp.upstreams.filesystem".into(),
            preview: serde_json::json!({"command": "npx"}),
        });

    assert_eq!(plan.package, "io.modelcontextprotocol/filesystem");
    assert_eq!(plan.steps.len(), 1);
    assert_eq!(plan.steps[0].kind, InstallStepKind::AddMcpUpstream);
}

#[test]
fn empty_plan_has_no_apply_steps() {
    let plan = InstallPlan::new("empty");
    assert!(plan.steps.is_empty());
}
```

Run:

```bash
cargo test -p agent-marketplace install_plan
```

Expected: FAIL because install-plan types do not exist yet.

- [ ] **Step 3: Export marketplace modules.**

Update `crates/agent-marketplace/src/lib.rs`:

```rust
mod conflict;
mod error;
mod plan;

pub use conflict::{InstallConflict, InstallConflictKind};
pub use error::{MarketplaceError, MarketplaceResult};
pub use plan::{InstallPlan, InstallStep, InstallStepKind};
```

- [ ] **Step 4: Implement error and conflict types.**

Create `crates/agent-marketplace/src/error.rs`:

```rust
use thiserror::Error;

pub type MarketplaceResult<T> = Result<T, MarketplaceError>;

#[derive(Debug, Error)]
pub enum MarketplaceError {
    #[error("invalid install target: {0}")]
    InvalidTarget(String),
    #[error("unsafe install parameter: {0}")]
    UnsafeParameter(String),
    #[error("install conflict: {0}")]
    Conflict(String),
}

impl MarketplaceError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::InvalidTarget(_) => "invalid_install_target",
            Self::UnsafeParameter(_) => "unsafe_install_parameter",
            Self::Conflict(_) => "install_conflict",
        }
    }
}
```

Create `crates/agent-marketplace/src/conflict.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallConflictKind {
    ExistingMcpUpstream,
    ExistingEnvVar,
    UnsupportedRuntime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallConflict {
    pub kind: InstallConflictKind,
    pub target: String,
    pub message: String,
}
```

- [ ] **Step 5: Implement install plan vocabulary.**

Create `crates/agent-marketplace/src/plan.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallPlan {
    pub package: String,
    #[serde(default)]
    pub steps: Vec<InstallStep>,
}

impl InstallPlan {
    pub fn new(package: impl Into<String>) -> Self {
        Self {
            package: package.into(),
            steps: Vec::new(),
        }
    }

    pub fn step(mut self, step: InstallStep) -> Self {
        self.steps.push(step);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstallStep {
    pub kind: InstallStepKind,
    pub description: String,
    pub target: String,
    pub preview: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallStepKind {
    AddMcpUpstream,
    SetEnvVar,
    VerifyRuntime,
}
```

- [ ] **Step 6: Verify install-plan tests.**

Run:

```bash
cargo test -p agent-marketplace install_plan
```

Expected: PASS.

### Task 2: Convert Registry Results Into Plans

**Files:**
- Create: `crates/agent-marketplace/src/mcp.rs`
- Test sidecar: `crates/agent-marketplace/src/mcp.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Write failing MCP plan tests.**

Add this `#[cfg(test)] mod tests` sidecar to `crates/agent-marketplace/src/mcp.rs`:

```rust
use super::*;
use agent_registry::{NormalizedMcpPackage, NormalizedMcpServer};

#[test]
fn creates_stdio_install_plan_from_npm_registry_package() {
    let server = NormalizedMcpServer {
        name: "io.modelcontextprotocol/filesystem".into(),
        description: Some("Filesystem MCP server".into()),
        latest_version: Some("0.6.2".into()),
        packages: vec![NormalizedMcpPackage {
            registry_type: "npm".into(),
            identifier: "@modelcontextprotocol/server-filesystem".into(),
            version: Some("0.6.2".into()),
            runtime_hint: Some("npx".into()),
            transport: Some("stdio".into()),
        }],
    };

    let plan = plan_mcp_server_install(&server).expect("plan created");
    assert_eq!(plan.package, "io.modelcontextprotocol/filesystem");
    assert_eq!(plan.steps[0].target, "mcp.upstreams.filesystem");
    assert_eq!(plan.steps[0].preview["command"], "npx");
}
```

- [ ] **Step 2: Export MCP planner.**

Update `crates/agent-marketplace/src/lib.rs`:

```rust
mod mcp;
pub use mcp::plan_mcp_server_install;
```

- [ ] **Step 3: Implement MCP plan generation.**

Create `crates/agent-marketplace/src/mcp.rs`:

```rust
use agent_registry::NormalizedMcpServer;

use crate::{InstallPlan, InstallStep, InstallStepKind, MarketplaceError, MarketplaceResult};

pub fn plan_mcp_server_install(server: &NormalizedMcpServer) -> MarketplaceResult<InstallPlan> {
    let package = server
        .packages
        .first()
        .ok_or_else(|| MarketplaceError::InvalidTarget("server has no installable packages".into()))?;
    let upstream_id = server
        .name
        .rsplit('/')
        .next()
        .unwrap_or(server.name.as_str())
        .replace('-', "_");
    let command = package.runtime_hint.as_deref().unwrap_or("npx");

    Ok(InstallPlan::new(&server.name).step(InstallStep {
        kind: InstallStepKind::AddMcpUpstream,
        description: format!("Add {upstream_id} MCP upstream"),
        target: format!("mcp.upstreams.{upstream_id}"),
        preview: serde_json::json!({
            "transport": "stdio",
            "command": command,
            "args": ["-y", package.identifier],
        }),
    }))
}
```

- [ ] **Step 4: Verify MCP plan tests.**

Run:

```bash
cargo test -p agent-marketplace mcp_plan
```

Expected: PASS.

### Task 3: Port MCP Install Safety Rules

**Files:**
- Create: `crates/agent-marketplace/src/mcp_params.rs`
- Modify: `crates/agent-marketplace/src/lib.rs`
- Test sidecar: `crates/agent-marketplace/src/mcp_params.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Read Lab install safety tests.**

Run:

```bash
rg -n "runtime_hint|stdio_argv|env_var_name|env_value|registry_url|docker_privileged|protected_env" ../lab/crates/lab/src/dispatch/marketplace/mcp_params.rs ../lab/crates/lab/src/dispatch/marketplace/mcp_dispatch.rs
```

Expected: AgentCast rejects unsafe runtime hints, dangerous arguments, protected env names, newline/null env values, and unsafe registry URLs.

- [ ] **Step 2: Write failing safety tests.**

Add this `#[cfg(test)] mod tests` sidecar to `crates/agent-marketplace/src/mcp_params.rs`:

```rust
use super::*;

#[test]
fn rejects_shell_runtime_hint() {
    let err = validate_runtime_hint("sh -c curl example.com | sh").unwrap_err();
    assert_eq!(err.kind(), "unsafe_install_parameter");
}

#[test]
fn rejects_protected_env_name() {
    let err = validate_env_name("PATH").unwrap_err();
    assert_eq!(err.kind(), "unsafe_install_parameter");
}

#[test]
fn rejects_env_value_with_newline_or_nul() {
    assert!(validate_env_value("hello\nworld").is_err());
    assert!(validate_env_value("hello\0world").is_err());
}
```

- [ ] **Step 3: Export safety helpers.**

Update `crates/agent-marketplace/src/lib.rs`:

```rust
mod mcp_params;
pub use mcp_params::{validate_env_name, validate_env_value, validate_runtime_hint};
```

- [ ] **Step 4: Implement safety helpers.**

Create `crates/agent-marketplace/src/mcp_params.rs`:

```rust
use crate::{MarketplaceError, MarketplaceResult};

const ALLOWED_RUNTIMES: &[&str] = &["npx", "node", "uvx", "python", "docker"];
const PROTECTED_ENV_NAMES: &[&str] = &["PATH", "HOME", "SHELL", "USER", "LD_PRELOAD"];

pub fn validate_runtime_hint(value: &str) -> MarketplaceResult<()> {
    if ALLOWED_RUNTIMES.contains(&value) {
        Ok(())
    } else {
        Err(MarketplaceError::UnsafeParameter(format!(
            "runtime hint `{value}` is not allowlisted"
        )))
    }
}

pub fn validate_env_name(value: &str) -> MarketplaceResult<()> {
    let valid_name = value
        .chars()
        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_');
    if !valid_name || PROTECTED_ENV_NAMES.contains(&value) {
        return Err(MarketplaceError::UnsafeParameter(format!(
            "env var `{value}` is not allowed"
        )));
    }
    Ok(())
}

pub fn validate_env_value(value: &str) -> MarketplaceResult<()> {
    if value.contains('\n') || value.contains('\0') {
        return Err(MarketplaceError::UnsafeParameter(
            "env value cannot contain newline or nul".into(),
        ));
    }
    Ok(())
}
```

- [ ] **Step 5: Verify safety tests.**

Run:

```bash
cargo test -p agent-marketplace mcp_install_safety
```

Expected: PASS.

### Task 4: Verify Full Marketplace Extraction

**Files:**
- Test sidecar: `crates/agent-marketplace/src/*.rs` (`#[cfg(test)] mod tests`)
- Read: `docs/plans/extract-crates/agent-marketplace.md`

- [ ] **Step 1: Run focused marketplace tests.**

Run:

```bash
cargo test -p agent-marketplace
```

Expected: PASS.

- [ ] **Step 2: Scan for post-v0 marketplace leakage.**

Run:

```bash
rg -n "acp|Acp|ACP|claude-plugin|LAB_|\\.lab|Plex|Sonarr|Radarr|Unraid|Gotify" crates/agent-marketplace
```

Expected: no output for v0 MCP install planning code.

- [ ] **Step 3: Commit the marketplace extraction slice.**

Run:

```bash
git add crates/agent-marketplace docs/plans/extract-crates/agent-marketplace.md
git commit -m "feat(marketplace): extract mcp install plans"
```

Expected: commit contains only `agent-marketplace` implementation, tests, and this plan if executing this slice alone.
