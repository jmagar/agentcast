---
title: "Gateway-First Skeleton Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/plans/extract-crates/gateway-first.md"
  - "docs/contracts/crates-and-dependencies.md"
related:
  - "docs/MVP.md"
  - "docs/TESTING.md"
  - "docs/plans/extract-crates/gateway-first.md"
last_reviewed: "2026-05-15"
last_modified: "2026-05-15"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "1290747"
review_basis: "superpowers writing-plans pass over gateway-first first-slice scope"
---

# Gateway-First Skeleton Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first working gateway skeleton: load configured local stdio MCP servers, expose normalized actions, route a selected action to an upstream call request, and validate the shared path through CLI-friendly services.

**Architecture:** This plan implements only the first mergeable gateway slice from `docs/plans/extract-crates/gateway-first.md`. `agent-protocol` owns shared models, `agent-config` loads and imports config, `agent-runtime` owns upstream snapshots/call primitives, and `agent-gateway` owns action projection/routing. Protected public MCP routes and upstream OAuth are required for v0 completion, but they should be implemented in separate follow-up plans because they cross `agent-auth`, `agent-store`, `agent-api`, and security-sensitive HTTP behavior.

**Tech Stack:** Rust 2024, `serde`, `serde_json`, `thiserror`, `toml`, source-side test sidecars, `cargo nextest`, `cargo xtask audit-docs`.

---

## Scope Check

The full gateway-first scope spans independent subsystems:

- first gateway skeleton: protocol models, config import, runtime snapshots, gateway action routing.
- protected public MCP routes: route policy, metadata, HTTP mounting, auth enforcement.
- upstream OAuth lifecycle: probe, PKCE/state, encrypted credential storage, refresh, runtime credential injection.

This plan covers only the first gateway skeleton. Write separate plans before implementing protected routes or OAuth.

## File Structure

- Modify: `crates/agent-protocol/src/lib.rs` - public exports for MCP launcher model modules.
- Create: `crates/agent-protocol/src/ids.rs` - stable ID newtypes.
- Create: `crates/agent-protocol/src/mcp.rs` - MCP server/config/capability models.
- Create: `crates/agent-protocol/src/action.rs` - normalized action and invocation models.
- Modify: `crates/agent-config/src/lib.rs` - public exports for config modules.
- Create: `crates/agent-config/src/error.rs` - config error type.
- Create: `crates/agent-config/src/mcp_json.rs` - MCP JSON/JSONC import parser.
- Create: `crates/agent-config/src/mcp_json/tests.rs` - source-side parser tests.
- Modify: `crates/agent-runtime/src/lib.rs` - public exports for runtime catalog and trait modules.
- Create: `crates/agent-runtime/src/catalog.rs` - runtime upstream snapshot types.
- Create: `crates/agent-runtime/src/upstream.rs` - `UpstreamCaller` trait and call request/result wrappers.
- Modify: `crates/agent-gateway/src/lib.rs` - public exports for catalog/routing modules.
- Create: `crates/agent-gateway/src/error.rs` - gateway error type.
- Create: `crates/agent-gateway/src/catalog.rs` - action projection and collision report.
- Create: `crates/agent-gateway/src/router.rs` - action route table.
- Create: `crates/agent-gateway/src/catalog/tests.rs` - source-side catalog tests.
- Create: `crates/agent-gateway/src/router/tests.rs` - source-side router tests.

## Task 1: Protocol Models

**Files:**
- Modify: `crates/agent-protocol/src/lib.rs`
- Create: `crates/agent-protocol/src/ids.rs`
- Create: `crates/agent-protocol/src/mcp.rs`
- Create: `crates/agent-protocol/src/action.rs`

- [ ] **Step 1: Replace the stub exports**

Write `crates/agent-protocol/src/lib.rs`:

```rust
pub mod action;
pub mod ids;
pub mod mcp;

pub use action::{LauncherAction, LauncherActionKind, ToolInvocation, ToolInvocationResult};
pub use ids::{LauncherActionId, McpServerId, McpToolId};
pub use mcp::{McpServerConfig, McpTransportConfig, ServerStatus};
```

- [ ] **Step 2: Add stable ID newtypes**

Write `crates/agent-protocol/src/ids.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct McpServerId(String);

impl McpServerId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for McpServerId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct McpToolId(String);

impl McpToolId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for McpToolId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct LauncherActionId(String);

impl LauncherActionId {
    pub fn from_server_tool(server_id: &McpServerId, tool_id: &McpToolId) -> Self {
        Self(format!("mcp:{}:{}", server_id.as_str(), tool_id.as_str()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LauncherActionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
```

- [ ] **Step 3: Add MCP config and status models**

Write `crates/agent-protocol/src/mcp.rs`:

```rust
use crate::ids::McpServerId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct McpServerConfig {
    pub id: McpServerId,
    pub name: String,
    pub enabled: bool,
    pub transport: McpTransportConfig,
    pub env_keys: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpTransportConfig {
    Stdio {
        command: String,
        args: Vec<String>,
        env: BTreeMap<String, String>,
    },
    StreamableHttp {
        url: String,
        bearer_token_env: Option<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerStatus {
    Disabled,
    Healthy,
    Degraded,
    Failed,
}
```

- [ ] **Step 4: Add action and invocation models**

Write `crates/agent-protocol/src/action.rs`:

```rust
use crate::ids::{LauncherActionId, McpServerId, McpToolId};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct LauncherAction {
    pub id: LauncherActionId,
    pub display_name: String,
    pub description: Option<String>,
    pub kind: LauncherActionKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LauncherActionKind {
    McpTool {
        server_id: McpServerId,
        tool_id: McpToolId,
    },
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ToolInvocation {
    pub action_id: LauncherActionId,
    pub arguments: Value,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ToolInvocationResult {
    pub action_id: LauncherActionId,
    pub output: Value,
}
```

- [ ] **Step 5: Run the protocol crate check**

Run:

```bash
cargo check -p agent-protocol
```

Expected: `Finished` with no compile errors.

- [ ] **Step 6: Commit**

Run:

```bash
git add crates/agent-protocol
git commit -m "feat: add launcher protocol models"
```

Expected: commit includes only `crates/agent-protocol`.

## Task 2: MCP JSON Import Parser

**Files:**
- Modify: `crates/agent-config/src/lib.rs`
- Create: `crates/agent-config/src/error.rs`
- Create: `crates/agent-config/src/mcp_json.rs`
- Create: `crates/agent-config/src/mcp_json/tests.rs`

- [ ] **Step 1: Add parser tests**

Write `crates/agent-config/src/mcp_json/tests.rs`:

```rust
use super::*;

#[test]
fn imports_stdio_servers_disabled_by_default() {
    let raw = r#"
    {
      "mcpServers": {
        "local": {
          "command": "agent",
          "args": ["serve"],
          "env": {"TOKEN": "secret"}
        }
      }
    }
    "#;

    let servers = parse_mcp_json(raw).expect("parse config");

    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "local");
    assert!(!servers[0].enabled);
    assert_eq!(servers[0].env_keys, vec!["TOKEN"]);
}

#[test]
fn jsonc_comments_do_not_strip_url_slashes_inside_strings() {
    let raw = r#"
    {
      // comment outside strings
      "mcpServers": {
        "remote": {
          "url": "https://example.test/mcp//stable"
        }
      }
    }
    "#;

    let servers = parse_mcp_json(raw).expect("parse config");

    match &servers[0].transport {
        agent_protocol::McpTransportConfig::StreamableHttp { url, .. } => {
            assert_eq!(url, "https://example.test/mcp//stable");
        }
        agent_protocol::McpTransportConfig::Stdio { .. } => panic!("expected streamable http"),
    }
}

#[test]
fn mcp_servers_object_wins_over_servers_object() {
    let raw = r#"
    {
      "mcpServers": {"preferred": {"command": "preferred"}},
      "servers": {"ignored": {"command": "ignored"}}
    }
    "#;

    let servers = parse_mcp_json(raw).expect("parse config");

    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "preferred");
}
```

- [ ] **Step 2: Wire source-side tests and public exports**

Write `crates/agent-config/src/lib.rs`:

```rust
pub mod error;
pub mod mcp_json;

pub use error::ConfigError;
pub use mcp_json::parse_mcp_json;
```

- [ ] **Step 3: Add config error type**

Write `crates/agent-config/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid MCP JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("MCP server `{name}` must define either command or url")]
    MissingTarget { name: String },
}
```

- [ ] **Step 4: Run tests to verify they fail**

Run:

```bash
cargo nextest run -p agent-config mcp_json
```

Expected: FAIL because `mcp_json` is not implemented yet.

- [ ] **Step 5: Implement the parser**

Write `crates/agent-config/src/mcp_json.rs`:

```rust
use crate::ConfigError;
use agent_protocol::{McpServerConfig, McpServerId, McpTransportConfig};
use serde_json::Value;
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

pub fn parse_mcp_json(raw: &str) -> Result<Vec<McpServerConfig>, ConfigError> {
    let stripped = strip_jsonc_comments(raw);
    let value: Value = serde_json::from_str(&stripped)?;
    let object = value.as_object().cloned().unwrap_or_default();
    let servers = object
        .get("mcpServers")
        .or_else(|| object.get("servers"))
        .or_else(|| object.get("mcp"))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let mut imported = Vec::new();
    for (name, spec) in servers {
        let server = parse_server(&name, &spec)?;
        imported.push(server);
    }
    imported.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(imported)
}

fn parse_server(name: &str, spec: &Value) -> Result<McpServerConfig, ConfigError> {
    let object = spec.as_object().cloned().unwrap_or_default();
    let env = object
        .get("env")
        .and_then(Value::as_object)
        .map(|values| {
            values
                .iter()
                .map(|(key, value)| (key.clone(), value.as_str().unwrap_or_default().to_string()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let env_keys = env.keys().cloned().collect::<Vec<_>>();

    if let Some(command) = object.get("command").and_then(Value::as_str) {
        let args = object
            .get("args")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        return Ok(McpServerConfig {
            id: McpServerId::new(name),
            name: name.to_string(),
            enabled: false,
            transport: McpTransportConfig::Stdio {
                command: command.to_string(),
                args,
                env,
            },
            env_keys,
        });
    }

    let url = ["url", "baseUrl", "base_url", "serverUrl", "server_url"]
        .iter()
        .find_map(|key| object.get(*key).and_then(Value::as_str));

    if let Some(url) = url {
        return Ok(McpServerConfig {
            id: McpServerId::new(name),
            name: name.to_string(),
            enabled: false,
            transport: McpTransportConfig::StreamableHttp {
                url: url.to_string(),
                bearer_token_env: object
                    .get("bearer_token_env")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
            },
            env_keys,
        });
    }

    Err(ConfigError::MissingTarget {
        name: name.to_string(),
    })
}

fn strip_jsonc_comments(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            output.push(ch);
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'/') {
            chars.next();
            for next in chars.by_ref() {
                if next == '\n' {
                    output.push('\n');
                    break;
                }
            }
            continue;
        }

        output.push(ch);
    }

    output
}
```

- [ ] **Step 6: Run parser tests**

Run:

```bash
cargo nextest run -p agent-config mcp_json
```

Expected: PASS for all `mcp_json` tests.

- [ ] **Step 7: Commit**

Run:

```bash
git add crates/agent-config
git commit -m "feat: import MCP JSON config"
```

Expected: commit includes only `crates/agent-config`.

## Task 3: Runtime Snapshot Boundary

**Files:**
- Modify: `crates/agent-runtime/src/lib.rs`
- Create: `crates/agent-runtime/src/catalog.rs`
- Create: `crates/agent-runtime/src/upstream.rs`

- [ ] **Step 1: Add runtime exports**

Write `crates/agent-runtime/src/lib.rs`:

```rust
pub mod catalog;
pub mod upstream;

pub use catalog::{RuntimeCatalogSnapshot, RuntimeTool};
pub use upstream::{ToolCallRequest, ToolCallResponse, UpstreamCaller};
```

- [ ] **Step 2: Add runtime catalog snapshot types**

Write `crates/agent-runtime/src/catalog.rs`:

```rust
use agent_protocol::{McpServerId, McpToolId, ServerStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RuntimeCatalogSnapshot {
    pub server_id: McpServerId,
    pub server_name: String,
    pub status: ServerStatus,
    pub tools: Vec<RuntimeTool>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RuntimeTool {
    pub id: McpToolId,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub input_schema: Value,
}
```

- [ ] **Step 3: Add upstream call trait**

Write `crates/agent-runtime/src/upstream.rs`:

```rust
use agent_protocol::{McpServerId, McpToolId};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct ToolCallRequest {
    pub server_id: McpServerId,
    pub tool_id: McpToolId,
    pub arguments: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolCallResponse {
    pub output: Value,
}

pub trait UpstreamCaller {
    type Error;

    fn call_tool(&self, request: ToolCallRequest) -> Result<ToolCallResponse, Self::Error>;
}
```

- [ ] **Step 4: Run the runtime crate check**

Run:

```bash
cargo check -p agent-runtime
```

Expected: `Finished` with no compile errors.

- [ ] **Step 5: Commit**

Run:

```bash
git add crates/agent-runtime
git commit -m "feat: add runtime catalog boundary"
```

Expected: commit includes only `crates/agent-runtime`.

## Task 4: Gateway Catalog And Router

**Files:**
- Modify: `crates/agent-gateway/src/lib.rs`
- Create: `crates/agent-gateway/src/error.rs`
- Create: `crates/agent-gateway/src/catalog.rs`
- Create: `crates/agent-gateway/src/catalog/tests.rs`
- Create: `crates/agent-gateway/src/router.rs`
- Create: `crates/agent-gateway/src/router/tests.rs`

- [ ] **Step 1: Add failing catalog tests**

Write `crates/agent-gateway/src/catalog/tests.rs`:

```rust
use super::*;
use agent_protocol::{McpServerId, McpToolId, ServerStatus};
use agent_runtime::{RuntimeCatalogSnapshot, RuntimeTool};
use serde_json::json;

#[test]
fn projects_runtime_tools_to_stable_actions() {
    let snapshot = RuntimeCatalogSnapshot {
        server_id: McpServerId::new("local"),
        server_name: "Local".to_string(),
        status: ServerStatus::Healthy,
        tools: vec![RuntimeTool {
            id: McpToolId::new("echo"),
            name: "echo".to_string(),
            title: Some("Echo".to_string()),
            description: Some("Return input".to_string()),
            input_schema: json!({"type": "object"}),
        }],
    };

    let catalog = GatewayCatalog::from_snapshots(vec![snapshot]);

    assert_eq!(catalog.actions.len(), 1);
    assert_eq!(catalog.actions[0].id.as_str(), "mcp:local:echo");
    assert_eq!(catalog.actions[0].display_name, "Echo");
    assert!(catalog.collisions.is_empty());
}

#[test]
fn reports_duplicate_action_ids() {
    let snapshots = vec![
        RuntimeCatalogSnapshot {
            server_id: McpServerId::new("local"),
            server_name: "Local".to_string(),
            status: ServerStatus::Healthy,
            tools: vec![RuntimeTool {
                id: McpToolId::new("echo"),
                name: "echo".to_string(),
                title: None,
                description: None,
                input_schema: json!({}),
            }],
        },
        RuntimeCatalogSnapshot {
            server_id: McpServerId::new("local"),
            server_name: "Local Duplicate".to_string(),
            status: ServerStatus::Healthy,
            tools: vec![RuntimeTool {
                id: McpToolId::new("echo"),
                name: "echo-alt".to_string(),
                title: None,
                description: None,
                input_schema: json!({}),
            }],
        },
    ];

    let catalog = GatewayCatalog::from_snapshots(snapshots);

    assert_eq!(catalog.actions.len(), 1);
    assert_eq!(catalog.collisions.len(), 1);
    assert_eq!(catalog.collisions[0].action_id.as_str(), "mcp:local:echo");
}
```

- [ ] **Step 2: Add failing router tests**

Write `crates/agent-gateway/src/router/tests.rs`:

```rust
use super::*;
use agent_protocol::{LauncherActionId, McpServerId, McpToolId};

#[test]
fn resolves_known_action_to_upstream_tool() {
    let route = ActionRoute {
        action_id: LauncherActionId::from_server_tool(
            &McpServerId::new("local"),
            &McpToolId::new("echo"),
        ),
        server_id: McpServerId::new("local"),
        tool_id: McpToolId::new("echo"),
    };
    let router = GatewayRouter::new(vec![route.clone()]);

    assert_eq!(router.resolve(&route.action_id), Some(&route));
}

#[test]
fn unknown_action_returns_none() {
    let router = GatewayRouter::new(Vec::new());
    let action_id =
        LauncherActionId::from_server_tool(&McpServerId::new("missing"), &McpToolId::new("echo"));

    assert!(router.resolve(&action_id).is_none());
}
```

- [ ] **Step 3: Wire gateway exports**

Write `crates/agent-gateway/src/lib.rs`:

```rust
pub mod catalog;
pub mod error;
pub mod router;

pub use catalog::{CollisionReport, GatewayCatalog};
pub use error::GatewayError;
pub use router::{ActionRoute, GatewayRouter};
```

- [ ] **Step 4: Add gateway error type**

Write `crates/agent-gateway/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("unknown action `{0}`")]
    UnknownAction(String),
}
```

- [ ] **Step 5: Run tests to verify they fail**

Run:

```bash
cargo nextest run -p agent-gateway catalog router
```

Expected: FAIL because catalog and router implementations do not exist yet.

- [ ] **Step 6: Implement catalog projection**

Write `crates/agent-gateway/src/catalog.rs`:

```rust
use agent_protocol::{LauncherAction, LauncherActionId, LauncherActionKind};
use agent_runtime::RuntimeCatalogSnapshot;
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GatewayCatalog {
    pub actions: Vec<LauncherAction>,
    pub collisions: Vec<CollisionReport>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollisionReport {
    pub action_id: LauncherActionId,
    pub existing_display_name: String,
    pub rejected_display_name: String,
}

impl GatewayCatalog {
    pub fn from_snapshots(snapshots: Vec<RuntimeCatalogSnapshot>) -> Self {
        let mut actions_by_id = BTreeMap::<LauncherActionId, LauncherAction>::new();
        let mut collisions = Vec::new();

        for snapshot in snapshots {
            for tool in snapshot.tools {
                let action_id = LauncherActionId::from_server_tool(&snapshot.server_id, &tool.id);
                let display_name = tool.title.clone().unwrap_or(tool.name.clone());
                let action = LauncherAction {
                    id: action_id.clone(),
                    display_name: display_name.clone(),
                    description: tool.description.clone(),
                    kind: LauncherActionKind::McpTool {
                        server_id: snapshot.server_id.clone(),
                        tool_id: tool.id.clone(),
                    },
                };

                if let Some(existing) = actions_by_id.get(&action_id) {
                    collisions.push(CollisionReport {
                        action_id,
                        existing_display_name: existing.display_name.clone(),
                        rejected_display_name: display_name,
                    });
                } else {
                    actions_by_id.insert(action_id, action);
                }
            }
        }

        Self {
            actions: actions_by_id.into_values().collect(),
            collisions,
        }
    }
}
```

- [ ] **Step 7: Implement router**

Write `crates/agent-gateway/src/router.rs`:

```rust
use agent_protocol::{LauncherActionId, McpServerId, McpToolId};
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionRoute {
    pub action_id: LauncherActionId,
    pub server_id: McpServerId,
    pub tool_id: McpToolId,
}

#[derive(Clone, Debug, Default)]
pub struct GatewayRouter {
    routes: BTreeMap<LauncherActionId, ActionRoute>,
}

impl GatewayRouter {
    pub fn new(routes: Vec<ActionRoute>) -> Self {
        Self {
            routes: routes
                .into_iter()
                .map(|route| (route.action_id.clone(), route))
                .collect(),
        }
    }

    pub fn resolve(&self, action_id: &LauncherActionId) -> Option<&ActionRoute> {
        self.routes.get(action_id)
    }
}
```

- [ ] **Step 8: Run gateway tests**

Run:

```bash
cargo nextest run -p agent-gateway catalog router
```

Expected: PASS for catalog and router tests.

- [ ] **Step 9: Commit**

Run:

```bash
git add crates/agent-gateway
git commit -m "feat: add gateway catalog routing"
```

Expected: commit includes only `crates/agent-gateway`.

## Task 5: Slice Verification

**Files:**
- Read: `docs/plans/extract-crates/gateway-first.md`
- Read: `docs/contracts/crates-and-dependencies.md`
- Read: `docs/TESTING.md`

- [ ] **Step 1: Run focused checks**

Run:

```bash
cargo nextest run -p agent-protocol
cargo nextest run -p agent-config
cargo nextest run -p agent-runtime
cargo nextest run -p agent-gateway
```

Expected: all focused crate tests pass.

- [ ] **Step 2: Run dependency-boundary grep**

Run:

```bash
rg -n "rmcp|agent_client_protocol|axum|tower|clap|rusqlite|sqlx" crates/agent-gateway crates/agent-auth crates/agent-runtime crates/agent-config
```

Expected: no matches in those crates for forbidden direct SDK/storage imports.

- [ ] **Step 3: Run docs audit**

Run:

```bash
cargo xtask audit-docs
```

Expected: `audit-docs: checked` with no errors.

- [ ] **Step 4: Run pre-push-sized verification**

Run:

```bash
cargo xtask ci
```

Expected: format check, workspace check, clippy, and nextest pass.

- [ ] **Step 5: Commit the plan if it is being included with the implementation**

Run:

```bash
git add docs/superpowers/plans/2026-05-15-gateway-first-skeleton.md
git commit -m "docs: add gateway skeleton implementation plan"
```

Expected: commit includes only this plan document unless implementation commits intentionally include it earlier.

## Self-Review

Spec coverage:

- Config discovery/import is covered by Task 2.
- Shared protocol models are covered by Task 1.
- Runtime snapshot and call boundary are covered by Task 3.
- Gateway action projection, collision reporting, and routing are covered by Task 4.
- Focused and boundary verification are covered by Task 5.
- Protected routes and OAuth are intentionally excluded from this first-slice plan and require separate plans.

Placeholder scan:

- This plan avoids placeholder markers and unnamed validation steps.
- Every code-changing step includes concrete code for the target file.

Type consistency:

- `McpServerId`, `McpToolId`, and `LauncherActionId` are defined in Task 1 and reused consistently in Tasks 2-4.
- `RuntimeCatalogSnapshot` and `RuntimeTool` are defined in Task 3 and reused consistently in Task 4.
