---
title: "agent-config Extraction Implementation Plan"
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
  - "docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-config Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract generic config loading, path resolution, and MCP upstream config models for AgentCast.

**Architecture:** `agent-config` owns reading and validating AgentCast config files plus explicit `.env` key references. Protocol lifecycle, runtime launch, and install behavior stay outside this crate.

**Tech Stack:** Rust 2024, serde, toml, url, directories.

---

## MVP Position

Config is the first extraction priority for the MCP launcher MVP.

## Lab Source Files

- `../lab/crates/lab/src/config.rs`
- `../lab/crates/lab/src/config/env_merge.rs`
- `../lab/crates/lab/src/dispatch/gateway/config.rs`
- `../lab/crates/lab/src/dispatch/gateway/config_mutation.rs`
- `../lab/config/acp-providers.docker.json`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab config source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`.
- MCP transport/config naming claims are cross-checked against `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md` and `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`.
- HTTP auth-adjacent config claims are cross-checked against `docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md`.

## Live Lab Findings

- `config.rs` contains `UpstreamConfig`, `ToolSearchConfig`, registry defaults, path resolution, secret redaction, and workspace path expansion.
- `config/env_merge.rs` is the strongest source for idempotent `.env` merge behavior, backups, conflict handling, comments, and 0600 permissions. AgentCast must restrict this to secrets, endpoint URLs, tokens, API keys, and runtime process env values.
- `dispatch/gateway/config.rs` has focused upstream insert/update/remove/validation tests that should drive `agent-config` behavior.
- `dispatch/gateway/config_mutation.rs` is mostly service credential mapping and should only be used as cautionary evidence.

## Extraction Boundary

Extract:

- app path resolution patterns.
- config file loading and restricted `.env` key-reference patterns.
- structured command plus args plus cwd plus explicit env binding model for local process providers.
- config mutation patterns for adding/removing upstreams.

Leave behind:

- Lab service defaults.
- `~/.lab` paths.
- ACP provider env names unless ACP is promoted.
- direct service credential discovery.
- arbitrary env overlays for non-secret settings.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-config/src/lib.rs` - public exports for config loading, paths, MCP upstreams, mutation, and env merge.
- Create: `crates/agent-config/src/error.rs` - crate-local error type with stable variants.
- Create: `crates/agent-config/src/paths.rs` - AgentCast path resolution and workspace expansion.
- Create: `crates/agent-config/src/mcp.rs` - MCP upstream config models and validation.
- Create: `crates/agent-config/src/load.rs` - config file loading, defaults, and explicit `.env` key reference resolution.
- Create: `crates/agent-config/src/mutation.rs` - add/update/remove helpers for upstream entries.
- Create: `crates/agent-config/src/env_merge.rs` - safe `.env` merge/write helper adapted from Lab.
- Add source-side test sidecars for: `crates/agent-config/src/{mcp,load}.rs` - MCP config load/validation tests.
- Add source-side test sidecars for: `crates/agent-config/src/mutation.rs` - mutation tests that preserve unrelated fields.
- Add source-side test sidecars for: `crates/agent-config/src/env_merge.rs` - idempotence, conflict, backup, and permission tests.

## Implementation Tasks

### Task 1: Define AgentCast Config Model

**Files:**
- Modify: `crates/agent-config/src/lib.rs`
- Create: `crates/agent-config/src/error.rs`
- Create: `crates/agent-config/src/paths.rs`
- Create: `crates/agent-config/src/mcp.rs`
- Create: `crates/agent-config/src/load.rs`
- Test sidecar: `crates/agent-config/src/{mcp,load}.rs`

- [ ] **Step 1: Inspect the Lab config model before naming AgentCast types.**

Run:

```bash
rg -n "struct UpstreamConfig|enum UpstreamTransport|struct ToolSearchConfig|fn config_dir|fn expand" ../lab/crates/lab/src/config.rs ../lab/crates/lab/src/dispatch/gateway/config.rs
```

Expected: Lab's reusable upstream shape is identified without carrying `~/.lab`, Lab service defaults, or Lab env names.

- [ ] **Step 2: Write the failing MCP config parsing test.**

Create a source-side test sidecar next to `crates/agent-config/src/{mcp,load}.rs` with:

```rust
use super::*;

#[test]
fn loads_stdio_mcp_upstream_with_structured_command() {
    let config = load_from_str(
        r#"
        [mcp.upstreams.filesystem]
        transport = "stdio"
        command = "npx"
        args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
        cwd = "/tmp"

        [mcp.upstreams.filesystem.env]
        ALLOW_WRITE = { source = "config", value = "false" }
        "#,
    )
    .expect("config parses");

    let upstream = config.mcp.upstreams.get("filesystem").expect("upstream exists");
    assert_eq!(upstream.id, "filesystem");
    assert!(matches!(upstream.transport, McpTransport::Stdio(_)));

    let McpTransport::Stdio(StdioUpstreamConfig { command, args, cwd, env }) = &upstream.transport else {
        panic!("expected stdio upstream");
    };
    assert_eq!(command, "npx");
    assert_eq!(args, &["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]);
    assert_eq!(cwd.as_deref(), Some("/tmp"));
    assert_eq!(
        env.get("ALLOW_WRITE"),
        Some(&EnvBinding::Config { value: "false".into() })
    );
}

#[test]
fn rejects_blank_upstream_id() {
    let err = load_from_str(
        r#"
        [mcp.upstreams.""]
        transport = "stdio"
        command = "node"
        "#,
    )
    .expect_err("blank upstream id is invalid");

    assert_eq!(err.kind(), "invalid_config");
    assert!(err.to_string().contains("upstream id"));
}
```

Run:

```bash
cargo nextest run -p agent-config mcp_config
```

Expected: FAIL because `load_from_str`, `McpTransport`, and `StdioUpstreamConfig` do not exist yet.

- [ ] **Step 3: Implement the minimal public exports and error type.**

Update `crates/agent-config/src/lib.rs`:

```rust
mod error;
mod load;
mod mcp;
mod paths;

pub use error::{ConfigError, ConfigResult};
pub use load::{load_from_str, AgentConfig};
pub use mcp::{EnvBinding, McpConfig, McpTransport, McpUpstreamConfig, StdioUpstreamConfig};
pub use paths::{AgentPaths, PathResolution};
```

Create `crates/agent-config/src/error.rs`:

```rust
use thiserror::Error;

pub type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    #[error("failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("failed to write config: {0}")]
    Io(#[from] std::io::Error),
}

impl ConfigError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::InvalidConfig(_) => "invalid_config",
            Self::Parse(_) => "parse_error",
            Self::Io(_) => "io_error",
        }
    }
}
```

- [ ] **Step 4: Implement MCP config models and validation.**

Create `crates/agent-config/src/mcp.rs`:

```rust
use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{ConfigError, ConfigResult};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpConfig {
    #[serde(default)]
    pub upstreams: BTreeMap<String, McpUpstreamConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpUpstreamConfig {
    #[serde(skip)]
    pub id: String,
    #[serde(flatten)]
    pub transport: McpTransport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub enum McpTransport {
    Stdio(StdioUpstreamConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StdioUpstreamConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub cwd: Option<PathBuf>,
    #[serde(default)]
    pub env: BTreeMap<String, EnvBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum EnvBinding {
    #[serde(rename = "config")]
    Config { value: String },
    #[serde(rename = "env")]
    Env {
        key: String,
        #[serde(default = "default_required")]
        required: bool,
    },
}

fn default_required() -> bool {
    true
}

impl McpConfig {
    pub fn validate(mut self) -> ConfigResult<Self> {
        for (id, upstream) in &mut self.upstreams {
            if id.trim().is_empty() {
                return Err(ConfigError::InvalidConfig("mcp upstream id cannot be blank".into()));
            }
            upstream.id = id.clone();
            upstream.validate()?;
        }
        Ok(self)
    }
}

impl McpUpstreamConfig {
    fn validate(&self) -> ConfigResult<()> {
        match &self.transport {
            McpTransport::Stdio(stdio) if stdio.command.trim().is_empty() => {
                Err(ConfigError::InvalidConfig(format!(
                    "mcp upstream `{}` command cannot be blank",
                    self.id
                )))
            }
            McpTransport::Stdio(stdio) => {
                for (name, binding) in &stdio.env {
                    if name.trim().is_empty() {
                        return Err(ConfigError::InvalidConfig("env name cannot be blank".into()));
                    }
                    if let EnvBinding::Env { key, .. } = binding {
                        if key.trim().is_empty() || key.chars().any(char::is_whitespace) {
                            return Err(ConfigError::InvalidConfig("env key cannot be blank or contain whitespace".into()));
                        }
                    }
                }
                Ok(())
            }
        }
    }
}
```

- [ ] **Step 5: Implement config loading.**

Create `crates/agent-config/src/load.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::{ConfigResult, McpConfig};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentConfig {
    #[serde(default)]
    pub mcp: McpConfig,
}

pub fn load_from_str(input: &str) -> ConfigResult<AgentConfig> {
    let parsed: AgentConfig = toml::from_str(input)?;
    Ok(AgentConfig {
        mcp: parsed.mcp.validate()?,
    })
}
```

- [ ] **Step 6: Add path resolution shell without product-specific paths.**

Create `crates/agent-config/src/paths.rs`:

```rust
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathResolution {
    Absolute(PathBuf),
    RelativeToConfig(PathBuf),
}

impl AgentPaths {
    pub fn resolve_config_relative(&self, value: impl AsRef<Path>) -> PathBuf {
        let path = value.as_ref();
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.config_dir.join(path)
        }
    }
}
```

- [ ] **Step 7: Verify the config model tests pass.**

Run:

```bash
cargo nextest run -p agent-config mcp_config
```

Expected: PASS for structured stdio config parsing and blank ID rejection.

### Task 2: Add Config Mutation Helpers

**Files:**
- Create: `crates/agent-config/src/mutation.rs`
- Modify: `crates/agent-config/src/lib.rs`
- Test sidecar: `crates/agent-config/src/mutation.rs`

- [ ] **Step 1: Inspect Lab gateway config mutation tests.**

Run:

```bash
rg -n "insert|update|remove|duplicate|unknown|preserve" ../lab/crates/lab/src/dispatch/gateway/config.rs ../lab/crates/lab/src/dispatch/gateway/config_mutation.rs
```

Expected: reusable add/update/remove behavior is identified, and service credential extraction is excluded.

- [ ] **Step 2: Write failing mutation tests.**

Create a source-side test sidecar next to `crates/agent-config/src/mutation.rs` with:

```rust
use super::*;

fn stdio(id: &str, command: &str) -> McpUpstreamConfig {
    McpUpstreamConfig {
        id: id.into(),
        transport: McpTransport::Stdio(StdioUpstreamConfig {
            command: command.into(),
            args: vec![],
            cwd: None,
            env: Default::default(),
        }),
    }
}

#[test]
fn adds_upstream_and_preserves_id() {
    let mut config = AgentConfig::default();
    add_mcp_upstream(&mut config, stdio("filesystem", "npx")).unwrap();
    assert!(config.mcp.upstreams.contains_key("filesystem"));
}

#[test]
fn rejects_duplicate_upstream_id() {
    let mut config = AgentConfig::default();
    add_mcp_upstream(&mut config, stdio("filesystem", "npx")).unwrap();
    let err = add_mcp_upstream(&mut config, stdio("filesystem", "node")).unwrap_err();
    assert_eq!(err.kind(), "invalid_config");
    assert!(err.to_string().contains("already exists"));
}

#[test]
fn remove_unknown_upstream_is_error() {
    let mut config = AgentConfig::default();
    let err = remove_mcp_upstream(&mut config, "missing").unwrap_err();
    assert_eq!(err.kind(), "invalid_config");
    assert!(err.to_string().contains("missing"));
}
```

- [ ] **Step 3: Export and implement mutation helpers.**

Update `crates/agent-config/src/lib.rs`:

```rust
mod mutation;
pub use mutation::{add_mcp_upstream, remove_mcp_upstream, update_mcp_upstream};
```

Create `crates/agent-config/src/mutation.rs`:

```rust
use crate::{AgentConfig, ConfigError, ConfigResult, McpUpstreamConfig};

pub fn add_mcp_upstream(config: &mut AgentConfig, upstream: McpUpstreamConfig) -> ConfigResult<()> {
    let id = upstream.id.trim();
    if id.is_empty() {
        return Err(ConfigError::InvalidConfig("mcp upstream id cannot be blank".into()));
    }
    if config.mcp.upstreams.contains_key(id) {
        return Err(ConfigError::InvalidConfig(format!("mcp upstream `{id}` already exists")));
    }
    config.mcp.upstreams.insert(id.to_string(), upstream);
    Ok(())
}

pub fn update_mcp_upstream(config: &mut AgentConfig, upstream: McpUpstreamConfig) -> ConfigResult<()> {
    let id = upstream.id.trim();
    if !config.mcp.upstreams.contains_key(id) {
        return Err(ConfigError::InvalidConfig(format!("mcp upstream `{id}` does not exist")));
    }
    config.mcp.upstreams.insert(id.to_string(), upstream);
    Ok(())
}

pub fn remove_mcp_upstream(config: &mut AgentConfig, id: &str) -> ConfigResult<McpUpstreamConfig> {
    config
        .mcp
        .upstreams
        .remove(id)
        .ok_or_else(|| ConfigError::InvalidConfig(format!("mcp upstream `{id}` does not exist")))
}
```

- [ ] **Step 4: Verify mutation tests.**

Run:

```bash
cargo nextest run -p agent-config config_mutation
```

Expected: PASS for add, duplicate, and remove behavior.

### Task 3: Port Environment Merge Safety

**Files:**
- Create: `crates/agent-config/src/env_merge.rs`
- Modify: `crates/agent-config/src/lib.rs`
- Test sidecar: `crates/agent-config/src/env_merge.rs`

- [ ] **Step 1: Read the Lab env merge safety cases.**

Run:

```bash
rg -n "idempotent|preserves_comments|skip_and_warn|force_overwrite|backup_pruning|unix_perms|restore" ../lab/crates/lab/src/config/env_merge.rs
```

Expected: AgentCast covers idempotence, comment preservation, conflicts, forced overwrite, backup retention, restore, and secure permissions without `LAB_*` names. This helper is only for secret values, endpoint URLs, tokens, API keys, and runtime process env values; durable non-secret settings stay in `config.toml`.

- [ ] **Step 2: Write failing env merge tests.**

Create a source-side test sidecar next to `crates/agent-config/src/env_merge.rs` with:

```rust
use super::*;

#[test]
fn merge_env_file_preserves_comments_and_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    std::fs::write(&path, "# local comment\nEXISTING=value\n").unwrap();

    let options = EnvMergeOptions {
        overwrite: false,
        backup: true,
    };
    merge_env_file(&path, [("AGENTCAST_TOKEN", "secret")], options).unwrap();
    merge_env_file(&path, [("AGENTCAST_TOKEN", "secret")], options).unwrap();

    let text = std::fs::read_to_string(path).unwrap();
    assert!(text.contains("# local comment"));
    assert_eq!(text.matches("AGENTCAST_TOKEN=secret").count(), 1);
    assert!(text.contains("EXISTING=value"));
}

#[test]
fn merge_env_file_does_not_overwrite_without_flag() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".env");
    std::fs::write(&path, "AGENTCAST_TOKEN=old\n").unwrap();

    let options = EnvMergeOptions {
        overwrite: false,
        backup: false,
    };
    let report = merge_env_file(&path, [("AGENTCAST_TOKEN", "new")], options).unwrap();
    assert_eq!(report.skipped, vec!["AGENTCAST_TOKEN"]);
    assert!(std::fs::read_to_string(path).unwrap().contains("AGENTCAST_TOKEN=old"));
}
```

- [ ] **Step 3: Export env merge types.**

Update `crates/agent-config/src/lib.rs`:

```rust
mod env_merge;
pub use env_merge::{merge_env_file, EnvMergeOptions, EnvMergeReport};
```

- [ ] **Step 4: Implement minimal idempotent env merge.**

Create `crates/agent-config/src/env_merge.rs`:

```rust
use std::collections::BTreeMap;
use std::path::Path;

use crate::ConfigResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvMergeOptions {
    pub overwrite: bool,
    pub backup: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EnvMergeReport {
    pub inserted: Vec<String>,
    pub updated: Vec<String>,
    pub skipped: Vec<String>,
}

pub fn merge_env_file<'a>(
    path: &Path,
    entries: impl IntoIterator<Item = (&'a str, &'a str)>,
    options: EnvMergeOptions,
) -> ConfigResult<EnvMergeReport> {
    let original = std::fs::read_to_string(path).unwrap_or_default();
    if options.backup && path.exists() {
        std::fs::write(path.with_extension("env.bak"), &original)?;
    }

    let mut values = BTreeMap::new();
    for (key, value) in entries {
        values.insert(key.to_string(), value.to_string());
    }

    let mut report = EnvMergeReport::default();
    let mut lines = Vec::new();
    let mut seen = BTreeMap::new();

    for line in original.lines() {
        if let Some((key, _value)) = line.split_once('=') {
            if let Some(new_value) = values.get(key) {
                seen.insert(key.to_string(), true);
                if options.overwrite {
                    lines.push(format!("{key}={new_value}"));
                    report.updated.push(key.to_string());
                } else {
                    lines.push(line.to_string());
                    report.skipped.push(key.to_string());
                }
                continue;
            }
        }
        lines.push(line.to_string());
    }

    for (key, value) in values {
        if !seen.contains_key(&key) {
            lines.push(format!("{key}={value}"));
            report.inserted.push(key);
        }
    }

    let mut output = lines.join("\n");
    output.push('\n');
    std::fs::write(path, output)?;
    Ok(report)
}
```

- [ ] **Step 5: Verify env merge tests.**

Run:

```bash
cargo nextest run -p agent-config env_merge
```

Expected: PASS for comment preservation, idempotence, and overwrite policy.

### Task 4: Verify Full Config Extraction

**Files:**
- Read: `docs/plans/extract-crates/agent-config.md`
- Read: `docs/reports/lab-extraction-source-map.md`
- Test sidecar: `crates/agent-config/src/*.rs`

- [ ] **Step 1: Run focused crate tests.**

Run:

```bash
cargo nextest run -p agent-config
```

Expected: PASS.

- [ ] **Step 2: Scan for Lab-specific names.**

Run:

```bash
rg -n "LAB_|\\.lab|Plex|Sonarr|Radarr|Unraid|Gotify" crates/agent-config
```

Expected: no output.

- [ ] **Step 3: Commit the config extraction slice.**

Run:

```bash
git add crates/agent-config docs/plans/extract-crates/agent-config.md
git commit -m "feat(config): extract mcp config foundation"
```

Expected: commit contains only `agent-config` implementation, tests, and this plan if executing this slice alone.
