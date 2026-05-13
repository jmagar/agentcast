---
title: "Observability"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/claude-code/markdown/0044-code-claude-com-docs-en-agent-sdk-observability.md"
  - "docs/references/claude-code/markdown/0088-code-claude-com-docs-en-monitoring-usage.md"
  - "docs/references/claude-code/markdown/0100-code-claude-com-docs-en-env-vars.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcp/docs/markdown/0113-modelcontextprotocol-io-specification-2025-03-26-server-utilities-logging.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Observability

AgentCast should expose enough runtime state for humans and agents to understand what happened without guessing.

## Principles

- more useful observability is better than hidden behavior.
- logs, events, health, and audit records are product surfaces.
- observability must be local/self-hosted by default.
- secret values are always redacted.
- machine-readable output matters as much as human-readable output.

## MVP Events

The MCP launcher MVP should emit structured events for:

- config discovery/import.
- server dedupe decisions.
- MCP server spawn/start/stop/restart.
- MCP initialize/list-tools/list-resources/list-prompts/read-resource/call-tool.
- catalog generation and collision handling.
- registry fetch/cache/index/search.
- install-plan preview/apply/verify.
- confirmation required/approved/denied.
- errors and retries.

## Agent-Friendly Requirements

CLI/API/MCP surfaces should allow agents to inspect:

- current server status.
- last startup error per server.
- discovered tools/resources/prompts per server.
- catalog freshness.
- registry cache freshness.
- active or recent invocations.
- audit records for sensitive operations.

Large event streams must support pagination/filtering. Summaries should be available without dumping full logs.

## Storage

MVP observability can use local files or SQLite. Future implementations may add richer stores, but core operation must not require hosted telemetry.

OpenTelemetry export can be added later as an optional integration. It must not become required for local operation.

## Redaction

Redaction must apply before persistence and before rendering through CLI/API/MCP/UI.

Use the canonical redaction literal from `docs/contracts/errors.md`.

Raw tool arguments, raw tool results, and upstream error strings may contain secrets or prompt-injection content. Persist them only behind explicit debug settings with redaction and size limits.

## Upstream References Checked

- `docs/references/mcp/docs/markdown/0113-modelcontextprotocol-io-specification-2025-03-26-server-utilities-logging.md`: MCP logging utility context.
- `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`: stdio stderr diagnostics and Streamable HTTP behavior.
- `docs/references/claude-code/markdown/0044-code-claude-com-docs-en-agent-sdk-observability.md`: agent observability comparison.
- `docs/references/claude-code/markdown/0088-code-claude-com-docs-en-monitoring-usage.md`: optional OpenTelemetry export comparison.
- `docs/references/claude-code/markdown/0100-code-claude-com-docs-en-env-vars.md`: raw tool detail logging is opt-in in Claude Code.
