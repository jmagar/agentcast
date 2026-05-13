---
title: "MCP Runtime Spec"
doc_type: "spec"
status: "active"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs:
  - "docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
  - "docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md"
  - "docs/references/mcp/repos/modelcontextprotocol-modelcontextprotocol.xml"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# MCP Runtime Spec

## Owning Crates

- `agent-config`: resolved MCP server config.
- `agent-runtime`: process lifecycle, server handles, catalog refresh orchestration.
- `agent-mcp`: MCP protocol client wrapper.
- `agent-gateway`: action catalog and routing.

## Module Shape

Suggested runtime modules:

```txt
crates/agent-runtime/src/process.rs
crates/agent-runtime/src/process_unix.rs
crates/agent-runtime/src/supervisor.rs
crates/agent-runtime/src/registry.rs
crates/agent-runtime/src/health.rs
crates/agent-runtime/src/error.rs
```

Suggested MCP adapter modules:

```txt
crates/agent-mcp/src/client.rs
crates/agent-mcp/src/stdio.rs
crates/agent-mcp/src/metadata.rs
crates/agent-mcp/src/error.rs
```

Suggested AgentCast MCP server modules:

```txt
crates/agent-mcp/src/server.rs
crates/agent-mcp/src/tools.rs
crates/agent-mcp/src/pagination.rs
```

## Startup Algorithm

For each enabled local stdio server:

1. validate `command` is non-empty.
2. validate `args` is an array of strings.
3. build the child environment from explicit config plus approved `.env` keys.
4. spawn child process without shell.
5. bind stdin/stdout to MCP transport.
6. initialize MCP client.
7. list tools.
8. list resources when supported.
9. list prompts when supported.
10. publish health.
11. hand tool/resource/prompt metadata to gateway.

## Shutdown Algorithm

1. stop accepting new invocations for the server.
2. attempt MCP/client close when supported.
3. close stdin.
4. wait for process exit until shutdown timeout.
5. kill process group if still alive and supported.
6. publish stopped or failed health.

## AgentCast MCP Tool Shape

List/search tools exposed by AgentCast's own MCP server should share a common request shape:

```rust
pub struct PageRequest {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub filter: Option<String>,
}
```

Responses should include:

```rust
pub struct Page<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<String>,
}
```

Initial MCP server tools should cover:

- `servers_list`
- `tools_list`
- `resources_list`
- `prompts_list`
- `resource_read`
- `tool_call`
- `actions_search`
- `registry_search`

Tool names can change during implementation, but the pagination/filtering behavior must not.

These names are AgentCast local draft tool names. The upstream MCP protocol methods remain `tools/list`, `tools/call`, `resources/list`, `resources/read`, `prompts/list`, and `prompts/get`.

## Verification

Run:

```bash
cargo test -p agent-runtime mcp
cargo test -p agent-mcp stdio
```

## Upstream References

- `docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md`
- `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`
- `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`
- `docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md`
- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`
- `docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md`
- `docs/references/mcp/repos/modelcontextprotocol-modelcontextprotocol.xml`
