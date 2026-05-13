---
title: "Launcher"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/fastmcp/docs/generate-cli.mdx"
  - "docs/references/mcp/docs/markdown/0027-modelcontextprotocol-io-seps-986-specify-format-for-tool-names.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
  - "docs/references/mcporter/docs/tool-calling.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Launcher

The launcher is the v0 product surface: configured MCP tools become searchable, invokable `LauncherAction` entries.

## Core Model

A `LauncherAction` is the normalized command-palette representation of one callable capability.

Required fields:

- `id`: stable AgentCast action id.
- `source`: owning source type and id.
- `upstream_tool_name`: original MCP tool name.
- `title`: human-facing label.
- `description`: short description.
- `input_schema`: JSON Schema for arguments.
- `category`: normalized grouping.
- `risk`: normalized risk label.
- `requires_confirmation`: whether invocation needs explicit confirmation.
- `search`: normalized searchable text.
- `metadata`: raw/source-specific details.

## Action IDs

MVP action ids use this shape:

```txt
mcp:<server-id>:tool:<tool-name>
```

Rules:

- `server-id` comes from AgentCast config.
- `tool-name` is the exact MCP tool name unless a collision policy creates an explicit exposed alias.
- IDs are lowercase where AgentCast owns the segment.
- upstream-owned names are preserved in metadata even if the exposed id is escaped.
- IDs must not change across process restarts for the same config and upstream tool.
- upstream MCP tool names are case-sensitive; AgentCast must not case-fold upstream names when routing calls.

## MCP Tool Mapping

Every discovered MCP tool maps to one launcher action.

Mapping:

- MCP server config key -> `source.id`.
- MCP tool `name` -> `upstream_tool_name`.
- MCP tool `title`, then `annotations.title`, then name -> `title`.
- MCP tool `description` -> `description`.
- MCP tool `inputSchema` -> `input_schema`.
- MCP annotations/destructive hints -> `risk` and `requires_confirmation` when available.

If MCP metadata is missing, AgentCast fills only safe defaults and keeps the raw tool metadata in `metadata`. MCP annotations are untrusted hints unless the server is trusted; they must not be the only basis for skipping confirmation.

## Categories

Initial categories:

- `files`
- `code`
- `search`
- `browser`
- `data`
- `system`
- `communication`
- `other`

Category inference must be deterministic. A user alias or curation overlay may override inferred category later.

## Risk Labels

Initial risk labels:

- `read`: reads local or remote data.
- `write`: mutates data or files.
- `execute`: executes commands or code.
- `network`: sends data to remote systems.
- `destructive`: deletes, overwrites, restarts, or otherwise causes high-impact change.
- `unknown`: risk cannot be inferred.

`destructive`, `execute`, and `unknown` actions require confirmation by default until policy says otherwise.

## Search Metadata

Search includes:

- action id.
- title.
- description.
- upstream server id.
- upstream tool name.
- category.
- aliases.
- tags.

Search must not index secrets, raw invocation arguments, or raw tool results.

## Later Metadata

Post-v0 additions:

- favorites.
- recent history.
- user aliases.
- custom categories.
- usage counts.
- per-project action visibility.

## Upstream References Checked

- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`: tool names, `title`, `inputSchema`, `outputSchema`, and annotation trust guidance.
- `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`: display precedence and `ToolAnnotations` fields.
- `docs/references/mcp/docs/markdown/0027-modelcontextprotocol-io-seps-986-specify-format-for-tool-names.md`: tool-name format is SEP context, not required as v0 validation policy.
- `docs/references/fastmcp/docs/generate-cli.mdx`: schema-to-interface generation comparison.
- `docs/references/mcporter/docs/tool-calling.md`: callable MCP tool UX patterns.
