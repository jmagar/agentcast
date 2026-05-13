---
title: "Launcher Action Spec"
doc_type: "spec"
status: "active"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs:
  - "docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md"
  - "docs/references/claude-code/markdown/0083-code-claude-com-docs-en-permissions.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Launcher Action Spec

## Owning Crates

- `agent-protocol`: `LauncherAction`, `LauncherActionId`, source metadata, invocation request/result models.
- `agent-mcp`: converts MCP tool metadata into protocol-neutral tool metadata.
- `agent-gateway`: merges tools into launcher actions, applies collision policy, routes invocations.
- `agent-runtime`: holds the current discovered catalog and invokes through gateway/MCP.

## Generation Flow

1. runtime starts enabled MCP servers.
2. `agent-mcp` lists tools.
3. gateway receives `(server_id, tool_metadata)` entries.
4. gateway creates candidate `action_id` values using `mcp:<server-id>:tool:<tool-name>`.
5. gateway detects collisions.
6. gateway emits final `LauncherAction` entries.
7. runtime stores the catalog snapshot.

## Collision Strategy

Initial implementation should fail validation on exposed `action_id` collision unless an explicit alias exists in `config.toml`.

Later options:

- explicit user alias.
- scoped non-MCP action families.
- user-controlled hidden action policy.

The collision strategy and risk taxonomy are AgentCast local policy. MCP tool annotations can inform defaults, but missing or optimistic annotations must not suppress confirmation requirements from the contract.

## Search Index

The MVP search index can be in-memory and rebuilt on discovery.

Index fields:

- action id.
- title.
- description.
- server id.
- upstream tool name.
- category.
- aliases.

## Verification

Run:

```bash
cargo test -p agent-protocol launcher
cargo test -p agent-gateway catalog
```

## Upstream References

- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`
- `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`
- `docs/references/claude-code/markdown/0083-code-claude-com-docs-en-permissions.md`
- `docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md`
