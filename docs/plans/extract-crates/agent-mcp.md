---
title: "agent-mcp Extraction Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/fastmcp/docs/client.mdx"
  - "docs/references/fastmcp/docs/inspecting.mdx"
  - "docs/references/fastmcp/docs/running.mdx"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
  - "docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md"
  - "docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md"
  - "docs/references/mcp/docs/markdown/0185-modelcontextprotocol-io-docs-learn-architecture.md"
  - "docs/references/mcporter/docs/livetests.md"
  - "docs/references/mcporter/docs/mcp.md"
  - "docs/references/mcporter/docs/tool-calling.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-mcp Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract MCP client/server lifecycle patterns for local upstream discovery and tool invocation.

**Architecture:** `agent-mcp` owns protocol adapters over RMCP. Process supervision for local stdio commands belongs in `agent-runtime`; normalized launcher actions belong in `agent-gateway`.

**Tech Stack:** Rust 2024, RMCP, Tokio, reqwest for streamable HTTP, serde_json.

---

## MVP Position

MCP is the protocol core of the launcher MVP.

## Lab Source Files

- `../lab/crates/lab/src/mcp.rs`
- `../lab/crates/lab/src/mcp/server.rs`
- `../lab/crates/lab/src/mcp/catalog.rs`
- `../lab/crates/lab/src/mcp/error.rs`
- `../lab/crates/lab/src/mcp/upstream.rs`
- `../lab/crates/lab/src/mcp/registry.rs`
- `../lab/crates/lab/src/dispatch/gateway/runtime.rs`
- `../lab/crates/lab/src/dispatch/gateway/types.rs`
- `../lab/crates/lab/examples/spike_rmcp_auth_client.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab MCP source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`; GatewayManager runtime impls are referenced through `crates/lab/src/dispatch/gateway/runtime.rs` in that snapshot.
- MCP lifecycle, stdio, Streamable HTTP, tools, resources, prompts, schemas, and HTTP authorization claims are cross-checked against `docs/references/mcp/docs/markdown/0185-modelcontextprotocol-io-docs-learn-architecture.md`, `docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md`, `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`, `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`, `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`, `docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md`, `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`, and `docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md`.
- mcporter and FastMCP are comparison references for client/tool-call and server-inspection ergonomics only: `docs/references/mcporter/docs/mcp.md`, `docs/references/mcporter/docs/tool-calling.md`, `docs/references/mcporter/docs/livetests.md`, `docs/references/fastmcp/docs/client.mdx`, `docs/references/fastmcp/docs/running.mdx`, and `docs/references/fastmcp/docs/inspecting.mdx`.

## Live Lab Findings

- `mcp/server.rs` is useful for RMCP server shape, prompts/resources/tools, completion, and error normalization; do not copy Lab service dispatch.
- `mcp/error.rs` and `mcp/upstream.rs` are the primary sources for MCP error classification and upstream result normalization.
- `mcp/catalog.rs` proves snapshots must include tool/resource/prompt visibility, but visibility policy belongs in `agent-gateway`.
- Local process launch comes from runtime/gateway sources, not `agent-mcp`.

## Extraction Boundary

Extract:

- MCP client connection lifecycle.
- tool/list and tool/call wrappers.
- resource and prompt discovery shapes when needed for catalog completeness.
- streamable HTTP client shape after stdio works.
- protocol error mapping into AgentCast errors.

Leave behind:

- Lab gateway catalog merge logic.
- Lab service-specific virtual servers.
- OAuth lifecycle until MCP OAuth is explicitly promoted.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-mcp/src/lib.rs` - public exports for client, metadata, errors, and transports.
- Create: `crates/agent-mcp/src/error.rs` - MCP protocol and transport error normalization.
- Create: `crates/agent-mcp/src/client.rs` - transport-neutral MCP client trait and owned client handle.
- Create: `crates/agent-mcp/src/stdio.rs` - stdio MCP connection adapter; process spawning is delegated to `agent-runtime`.
- Create: `crates/agent-mcp/src/metadata.rs` - MCP tool/resource/prompt metadata conversion.
- Create: `crates/agent-mcp/src/result.rs` - normalized tool call result model.
- Add source-side test sidecars for: `crates/agent-mcp/src/metadata.rs` - tool metadata conversion tests.
- Add source-side test sidecars for: `crates/agent-mcp/src/error.rs` - upstream error mapping tests.
- Add source-side test sidecars for: `crates/agent-mcp/src/stdio.rs` - fixture-backed stdio client tests.

## Implementation Tasks

### Task 1: Implement Local Stdio MCP Client Wrapper

**Files:**
- Modify: `crates/agent-mcp/src/lib.rs`
- Create: `crates/agent-mcp/src/error.rs`
- Create: `crates/agent-mcp/src/client.rs`
- Create: `crates/agent-mcp/src/stdio.rs`
- Test sidecar: `crates/agent-mcp/src/stdio.rs`

- [ ] **Step 1: Inspect Lab's MCP server and upstream boundary.**

Run:

```bash
rg -n "list_tools|call_tool|initialize|upstream|stdio|ToolResult|CallTool" ../lab/crates/lab/src/mcp ../lab/crates/lab/src/dispatch/gateway/runtime.rs
```

Expected: `agent-mcp` owns protocol calls and result normalization; `agent-runtime` owns subprocess lifecycle.

- [ ] **Step 2: Write failing stdio client tests with a fixture server.**

Create a source-side test sidecar next to `crates/agent-mcp/src/stdio.rs` with:

```rust
use super::*;

#[tokio::test]
async fn stdio_client_lists_tools_from_fixture_server() {
    let server = std::env::current_dir()
        .unwrap()
        .join("src/fixtures/mcp_echo_server.js");

    let connection = StdioConnection::new("node", [server.display().to_string()]);
    let client = McpClient::connect_stdio(connection, McpClientOptions::default())
        .await
        .expect("fixture client connects");

    let tools = client.list_tools().await.expect("tools listed");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "echo");
}

#[tokio::test]
async fn stdio_client_calls_fixture_tool() {
    let server = std::env::current_dir()
        .unwrap()
        .join("src/fixtures/mcp_echo_server.js");

    let connection = StdioConnection::new("node", [server.display().to_string()]);
    let client = McpClient::connect_stdio(connection, McpClientOptions::default())
        .await
        .expect("fixture client connects");

    let result = client
        .call_tool("echo", serde_json::json!({"message": "hello"}))
        .await
        .expect("tool call succeeds");

    assert_eq!(result.text(), Some("hello"));
}
```

Run:

```bash
cargo nextest run -p agent-mcp stdio_client
```

Expected: FAIL because the stdio wrapper and fixture server do not exist yet.

- [ ] **Step 3: Add public client and error exports.**

Update `crates/agent-mcp/src/lib.rs`:

```rust
mod client;
mod error;
mod result;
mod stdio;

pub use client::{McpClient, McpClientOptions};
pub use error::{McpError, McpResult};
pub use result::{McpToolContent, McpToolResult};
pub use stdio::StdioConnection;
```

Create `crates/agent-mcp/src/error.rs`:

```rust
use thiserror::Error;

pub type McpResult<T> = Result<T, McpError>;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("mcp connection failed: {0}")]
    Connection(String),
    #[error("mcp protocol error: {0}")]
    Protocol(String),
    #[error("mcp tool error: {0}")]
    Tool(String),
}

impl McpError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Connection(_) => "mcp_connection",
            Self::Protocol(_) => "mcp_protocol",
            Self::Tool(_) => "mcp_tool",
        }
    }
}
```

- [ ] **Step 4: Add transport-neutral result and client handles.**

Create `crates/agent-mcp/src/result.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolResult {
    #[serde(default)]
    pub content: Vec<McpToolContent>,
    #[serde(default)]
    pub is_error: bool,
    #[serde(default)]
    pub raw: serde_json::Value,
}

impl McpToolResult {
    pub fn text(&self) -> Option<&str> {
        self.content.iter().find_map(|content| match content {
            McpToolContent::Text { text } => Some(text.as_str()),
            McpToolContent::Unknown { .. } => None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpToolContent {
    Text { text: String },
    Unknown { raw: serde_json::Value },
}
```

Create `crates/agent-mcp/src/client.rs`:

```rust
use serde_json::Value;

use crate::{McpResult, McpToolResult, StdioConnection};

#[derive(Debug, Clone, Default)]
pub struct McpClientOptions {
    pub initialize_timeout_ms: Option<u64>,
    pub request_timeout_ms: Option<u64>,
}

#[derive(Debug)]
pub struct McpClient {
    inner: McpClientInner,
}

#[derive(Debug)]
enum McpClientInner {
    Stdio(crate::stdio::StdioClient),
}

impl McpClient {
    pub async fn connect_stdio(
        connection: StdioConnection,
        options: McpClientOptions,
    ) -> McpResult<Self> {
        let client = crate::stdio::StdioClient::connect(connection, options).await?;
        Ok(Self {
            inner: McpClientInner::Stdio(client),
        })
    }

    pub async fn list_tools(&self) -> McpResult<Vec<crate::metadata::McpToolMetadata>> {
        match &self.inner {
            McpClientInner::Stdio(client) => client.list_tools().await,
        }
    }

    pub async fn call_tool(&self, name: &str, args: Value) -> McpResult<McpToolResult> {
        match &self.inner {
            McpClientInner::Stdio(client) => client.call_tool(name, args).await,
        }
    }
}
```

- [ ] **Step 5: Implement stdio connection shell around RMCP.**

Create `crates/agent-mcp/src/stdio.rs`:

```rust
use serde_json::Value;

use crate::{McpClientOptions, McpError, McpResult, McpToolResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdioConnection {
    pub command: String,
    pub args: Vec<String>,
}

impl StdioConnection {
    pub fn new(command: impl Into<String>, args: impl IntoIterator<Item = String>) -> Self {
        Self {
            command: command.into(),
            args: args.into_iter().collect(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct StdioClient {
    upstream_id: String,
}

impl StdioClient {
    pub async fn connect(connection: StdioConnection, _options: McpClientOptions) -> McpResult<Self> {
        if connection.command.trim().is_empty() {
            return Err(McpError::Connection("stdio command cannot be blank".into()));
        }
        Ok(Self {
            upstream_id: connection.command,
        })
    }

    pub async fn list_tools(&self) -> McpResult<Vec<crate::metadata::McpToolMetadata>> {
        Err(McpError::Protocol(format!(
            "stdio RMCP integration for `{}` is not implemented yet",
            self.upstream_id
        )))
    }

    pub async fn call_tool(&self, name: &str, _args: Value) -> McpResult<McpToolResult> {
        Err(McpError::Protocol(format!(
            "stdio tool `{name}` call is not implemented yet"
        )))
    }
}
```

Expected: this step creates the public seam; the next execution pass replaces the stub internals with RMCP calls before tests pass.

### Task 2: Add Protocol Metadata Conversion

**Files:**
- Modify: `crates/agent-mcp/src/lib.rs`
- Create: `crates/agent-mcp/src/metadata.rs`
- Test sidecar: `crates/agent-mcp/src/metadata.rs`

- [ ] **Step 1: Write failing metadata conversion tests.**

Create a source-side test sidecar next to `crates/agent-mcp/src/metadata.rs` with:

```rust
use super::*;
use serde_json::json;

#[test]
fn converts_raw_mcp_tool_to_agent_metadata() {
    let raw = RawMcpTool {
        name: "read_file".into(),
        title: Some("Read file".into()),
        description: Some("Read a local file".into()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {"type": "string"}
            },
            "required": ["path"]
        }),
        annotations: json!({"destructiveHint": false}),
    };

    let metadata = McpToolMetadata::from_raw("filesystem", raw);
    assert_eq!(metadata.upstream_id, "filesystem");
    assert_eq!(metadata.name, "read_file");
    assert_eq!(metadata.action_id, "mcp:filesystem:tool:read_file");
    assert_eq!(metadata.title.as_deref(), Some("Read file"));
    assert_eq!(metadata.input_schema["required"][0], "path");
}
```

- [ ] **Step 2: Export metadata types.**

Update `crates/agent-mcp/src/lib.rs`:

```rust
mod metadata;
pub use metadata::{McpToolMetadata, RawMcpTool};
```

- [ ] **Step 3: Implement metadata conversion.**

Create `crates/agent-mcp/src/metadata.rs`:

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawMcpTool {
    pub name: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub input_schema: Value,
    #[serde(default)]
    pub annotations: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolMetadata {
    pub upstream_id: String,
    pub name: String,
    pub action_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub input_schema: Value,
    pub annotations: Value,
}

impl McpToolMetadata {
    pub fn from_raw(upstream_id: impl Into<String>, raw: RawMcpTool) -> Self {
        let upstream_id = upstream_id.into();
        Self {
            action_id: format!("mcp:{}:tool:{}", upstream_id, raw.name),
            upstream_id,
            name: raw.name,
            title: raw.title,
            description: raw.description,
            input_schema: raw.input_schema,
            annotations: raw.annotations,
        }
    }
}
```

- [ ] **Step 4: Verify metadata tests.**

Run:

```bash
cargo nextest run -p agent-mcp metadata
```

Expected: PASS for raw tool metadata conversion.

### Task 3: Port MCP Error Normalization

**Files:**
- Create or modify: `crates/agent-mcp/src/error.rs`
- Test sidecar: `crates/agent-mcp/src/error.rs`

- [ ] **Step 1: Read Lab MCP error cases.**

Run:

```bash
sed -n '1,260p' ../lab/crates/lab/src/mcp/error.rs
rg -n "extract_error_info|normalize_upstream_result|preserves_user_errors|poison" ../lab/crates/lab/src/mcp/server.rs ../lab/crates/lab/src/mcp/upstream.rs
```

Expected: AgentCast preserves stable MCP error kinds and does not mark server health failed for user-level invocation errors.

- [ ] **Step 2: Write failing error mapping tests.**

Create a source-side test sidecar next to `crates/agent-mcp/src/error.rs` with:

```rust
use super::*;

#[test]
fn user_tool_error_is_not_connection_error() {
    let error = normalize_tool_error("permission denied by tool");
    assert_eq!(error.kind(), "mcp_tool");
    assert!(matches!(error, McpError::Tool(_)));
}

#[test]
fn transport_eof_is_connection_error() {
    let error = McpError::Connection("server closed stdout".into());
    assert_eq!(error.kind(), "mcp_connection");
}
```

- [ ] **Step 3: Implement normalization helper.**

Update `crates/agent-mcp/src/lib.rs`:

```rust
pub use error::{normalize_tool_error, McpError, McpResult};
```

Update `crates/agent-mcp/src/error.rs`:

```rust
pub fn normalize_tool_error(message: impl Into<String>) -> McpError {
    McpError::Tool(message.into())
}
```

- [ ] **Step 4: Verify error mapping tests.**

Run:

```bash
cargo nextest run -p agent-mcp error_mapping
```

Expected: PASS for tool-level and connection-level error classification.

### Task 4: Wire RMCP Internals Behind The Public Boundary

**Files:**
- Modify: `crates/agent-mcp/src/stdio.rs`
- Modify: `crates/agent-mcp/src/client.rs`
- Test sidecar: `crates/agent-mcp/src/stdio.rs`

- [ ] **Step 1: Check the installed RMCP API against Lab usage.**

Run:

```bash
rg -n "rmcp|ServiceExt|serve_client|TokioChildProcess|list_tools|call_tool" Cargo.toml crates ../lab/crates/lab/src/mcp ../lab/crates/lab/examples
```

Expected: the concrete RMCP symbols for connecting stdio, initializing, listing tools, and calling tools are known before replacing the stub.

- [ ] **Step 2: Replace stdio stubs with RMCP calls.**

Expected implementation shape in `crates/agent-mcp/src/stdio.rs`:

```rust
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParams, JsonObject},
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use tokio::process::Command;

impl StdioClient {
    pub async fn connect(connection: StdioConnection, options: McpClientOptions) -> McpResult<Self> {
        let mut command = Command::new(&connection.command);
        command.configure(|cmd| {
            cmd.args(&connection.args);
        });

        let transport = TokioChildProcess::new(command)
            .map_err(|error| McpError::Connection(error.to_string()))?;
        let peer = ().serve(transport).await
            .map_err(|error| McpError::Protocol(error.to_string()))?;

        Ok(Self { peer, options })
    }

    pub async fn list_tools(&self) -> McpResult<Vec<crate::metadata::McpToolMetadata>> {
        let tools = self.peer.list_all_tools().await
            .map_err(|error| McpError::Protocol(error.to_string()))?;

        Ok(tools
            .into_iter()
            .map(|tool| crate::metadata::McpToolMetadata::from_rmcp_tool(&self.upstream_id, tool))
            .collect())
    }

    pub async fn call_tool(&self, name: &str, args: Value) -> McpResult<McpToolResult> {
        let arguments: JsonObject = serde_json::from_value(args)
            .map_err(|error| McpError::Tool(error.to_string()))?;
        let result = self.peer.call_tool(
            CallToolRequestParams::new(name.to_string()).with_arguments(arguments),
        ).await.map_err(|error| McpError::Tool(error.to_string()))?;

        Ok(McpToolResult::from_rmcp(result))
    }
}
```

Execution note: use the exact workspace RMCP imports from Step 1. Do not introduce a second process-spawn abstraction here; `agent-runtime` owns long-running process supervision.

- [ ] **Step 3: Verify stdio client tests.**

Run:

```bash
cargo nextest run -p agent-mcp stdio_client
```

Expected: PASS against the fixture MCP server.

### Task 5: Verify Full MCP Extraction

**Files:**
- Read: `docs/plans/extract-crates/agent-mcp.md`
- Test sidecar: `crates/agent-mcp/src/*.rs`

- [ ] **Step 1: Run focused MCP tests.**

Run:

```bash
cargo nextest run -p agent-mcp
```

Expected: PASS.

- [ ] **Step 2: Scan for Lab service leakage.**

Run:

```bash
rg -n "Plex|Sonarr|Radarr|Unraid|Gotify|LAB_|\\.lab|ServiceCatalog" crates/agent-mcp
```

Expected: no output.

- [ ] **Step 3: Commit the MCP extraction slice.**

Run:

```bash
git add crates/agent-mcp docs/plans/extract-crates/agent-mcp.md
git commit -m "feat(mcp): extract protocol client foundation"
```

Expected: commit contains only `agent-mcp` implementation, tests, fixtures, and this plan if executing this slice alone.
