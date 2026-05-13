---
title: "Launcher Action Contract"
doc_type: "contract"
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

# Launcher Action Contract

This contract defines stable `LauncherAction` behavior for the MCP launcher MVP.

## Required Fields

Every launcher action must include:

- `action_id`
- `source.kind`
- `source.server_id`
- `upstream_tool_name`
- `title`
- `description`
- `input_schema`
- `category`
- `risk`
- `requires_confirmation`

## ID Rules

MCP action ids must use:

```txt
mcp:<server-id>:tool:<tool-name>
```

Requirements:

- IDs are stable for the same config and upstream tool.
- source server id must match AgentCast config.
- original MCP tool name must be preserved.
- implementations must preserve `action_id` exactly across CLI, API, gateway, search, history, and install-plan references.
- collisions must be reported or resolved by explicit alias policy.
- collisions must not be silently hidden by generating an unrequested id.
- alias policy must be explicit config, and alias actions must keep source traceability to the original server/tool.

## Risk Rules

Actions with risk `destructive`, `execute`, or `unknown` must require confirmation by default.

Risk labels are AgentCast local policy. Upstream MCP tool annotations are hints and must not be treated as trusted proof of safety.

## Source Traceability

Every action must be traceable back to:

- configured server id.
- transport.
- original MCP tool name.
- raw MCP metadata.
- generated or user-configured alias, when present.

## Acceptance Tests

Implementations must test:

- stable id generation.
- one MCP tool produces one launcher action.
- missing metadata gets safe defaults.
- collisions are deterministic and visible.
- confirmation is required for destructive, execute, and unknown risk actions.

## Upstream References

- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`
- `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`
- `docs/references/claude-code/markdown/0083-code-claude-com-docs-en-permissions.md`
- `docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md`
