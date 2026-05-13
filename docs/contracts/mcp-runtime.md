---
title: "MCP Runtime Contract"
doc_type: "contract"
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

# MCP Runtime Contract

This contract defines required v0 behavior for configured MCP servers.

## Required Lifecycle

For each enabled server, runtime must:

1. validate resolved config.
2. start or connect transport.
3. initialize MCP.
4. discover tools.
5. discover resources when supported.
6. discover prompts when supported.
7. publish health.
8. expose initialized client handles and discovered metadata to gateway.
9. shut down cleanly.

## MCP Surface Requirements

When exposed through AgentCast's MCP server, list/search tools must:

- accept pagination inputs.
- accept filtering inputs where the collection can be large.
- apply filtering before pagination.
- avoid unbounded default output.
- return continuation metadata when more results exist.
- delegate to the same runtime/gateway operations as CLI and API.

These AgentCast MCP tools are local product tools. They must coexist with, but do not replace, upstream MCP protocol operations such as `tools/list`, `resources/list`, and `prompts/list`.

## Local Stdio Rules

Requirements:

- command and args are structured fields.
- runtime must not shell-concatenate command strings.
- stderr is captured or explicitly discarded with a documented reason.
- child process ownership is tracked.
- runtime environment comes from explicit config plus approved `.env` keys; the full parent environment must not be inherited by default.
- shutdown is explicit and observable.

## Health Requirements

Server health must include:

- server id.
- transport.
- state.
- discovered tool count.
- discovered resource count when supported.
- discovered prompt count when supported.
- last error kind and message when failed.

## Timeout Requirements

Runtime must apply timeouts to:

- spawn.
- initialize.
- discovery.
- resource read.
- invocation.
- shutdown.

Timeouts must return normalized errors.

## Acceptance Tests

Implementations must test:

- successful local stdio initialize/discovery using a fixture server.
- tool/resource listing using a fixture server.
- prompt listing using a fixture server.
- MCP surface pagination/filtering for list tools.
- resource read using a fixture server.
- spawn failure.
- initialize timeout.
- invocation timeout.
- clean shutdown.
- normalized error mapping.

## Upstream References

- `docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md`
- `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`
- `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`
- `docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md`
- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`
- `docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md`
- `docs/references/mcp/repos/modelcontextprotocol-modelcontextprotocol.xml`
