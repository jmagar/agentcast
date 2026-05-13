---
title: "MCP Runtime"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/mcp/docs/markdown/0035-modelcontextprotocol-io-specification-2025-06-18-basic-lifecycle.md"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
  - "docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# MCP Runtime

This document defines v0 runtime behavior for configured MCP servers.

The v0 protocol baseline is MCP 2025-11-25 behavior where practical, with local stdio required first. Streamable HTTP is the next transport; the older HTTP+SSE transport is compatibility-only and not the primary target.

## Lifecycle

For each enabled MCP server:

1. load resolved config from `agent-config`.
2. validate transport settings.
3. spawn or connect transport.
4. initialize MCP client.
5. send the initialized notification when required by the client implementation.
6. list tools.
7. list resources when supported.
8. list prompts when supported.
9. publish server health.
10. expose tools/resources/prompts to gateway/catalog.
11. route invocations and resource reads.
12. shut down cleanly on request or process exit.

## AgentCast MCP Surface

The AgentCast MCP server surface must be thin over runtime/gateway behavior.

List/search tools exposed by AgentCast's MCP server must support pagination and filtering for large collections, including servers, tools, resources, prompts, actions, registry candidates, runtime health, and observability events.

Rules:

- filtering applies before pagination.
- no list/search MCP tool returns unbounded output by default.
- responses include continuation metadata when more results exist.
- detail-heavy content should be fetched through explicit read/get tools.
- output should be stable enough for LLM agents to call repeatedly without prompt-specific parsing.

## Supported Transports

v0 required:

- `local-stdio`

Next:

- `streamable-http`

Reserved:

- `ssh-stdio`
- Docker
- Kubernetes
- AgentCast node runtime

## Local Stdio Spawn

Runtime must spawn local stdio servers from structured config:

```toml
[mcp.servers.filesystem]
transport = "local-stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
env = {}
enabled = true
```

Rules:

- command and args are structured, not shell strings.
- runtime does not invoke through a shell unless a future config field explicitly opts in.
- child stdin/stdout are owned by the MCP transport.
- stderr is captured for diagnostics with length limits, but stderr output alone is not treated as protocol failure.
- shutdown attempts graceful protocol/transport close before process kill.

## Health States

Initial server states:

- `disabled`
- `starting`
- `ready`
- `degraded`
- `failed`
- `stopping`
- `stopped`

Health includes:

- server id.
- transport.
- state.
- last successful initialize timestamp.
- discovered tool count.
- discovered resource count when supported.
- discovered prompt count when supported.
- last error kind/message.

## Timeouts

Initial timeout categories:

- spawn timeout.
- initialize timeout.
- list tools timeout.
- list resources timeout.
- list prompts timeout.
- read resource timeout.
- invocation timeout.
- shutdown timeout.

Timeouts are config-driven with conservative defaults. A timeout must produce a normalized error rather than a hung command.

## Cancellation

Invocation cancellation must:

- stop waiting for the tool result.
- attempt protocol-level cancellation when supported; for non-task MCP requests this means `notifications/cancelled`, while task-augmented execution uses task-specific cancellation.
- preserve the server process unless the process is unhealthy.
- emit a normalized cancellation error.

## Restart

Runtime may restart failed servers explicitly.

Automatic restart is post-v0 unless added with backoff and storm protection.

## Error Normalization

Runtime maps protocol/process failures into the error taxonomy in `docs/ERRORS.md`.

Examples:

- process spawn failure -> `runtime.spawn_failed`.
- initialize timeout -> `mcp.initialize_timeout`.
- tool list decode failure -> `mcp.discovery_failed`.
- resource list/read failure -> `mcp.resource_failed`.
- tool invocation error -> `mcp.tool_failed`.

## Streamable HTTP Later

Streamable HTTP must reuse the same server lifecycle and health vocabulary. The transport adapter changes; gateway/catalog behavior does not.

Streamable HTTP clients must send the required `Accept` headers and handle JSON or SSE responses. Local Streamable HTTP servers should follow upstream Origin validation, localhost binding, and authentication guidance.

## Upstream References Checked

- `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`: stdio, Streamable HTTP, HTTP+SSE compatibility, headers, stderr, and security requirements.
- `docs/references/mcp/docs/markdown/0035-modelcontextprotocol-io-specification-2025-06-18-basic-lifecycle.md`: initialize/initialized lifecycle compatibility context.
- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`: `tools/list`, `tools/call`, and task-support metadata.
- `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`: resource listing/read behavior.
- `docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md`: prompt listing/get behavior.
- `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`: `notifications/cancelled` and task cancellation distinction.
