---
title: "Security"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/claude-code/markdown/0096-code-claude-com-docs-en-security.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
  - "docs/references/mcp/docs/markdown/0120-modelcontextprotocol-io-docs-tutorials-security-security-best-practices.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Security

AgentCast v0 runs local MCP servers. Local stdio MCP servers are executable code and must be treated as trusted programs selected by the user.

## Trust Model

v0 assumes:

- the user controls local AgentCast config.
- configured local stdio servers are trusted to run as the user.
- registry metadata is untrusted until reviewed.
- MCP tool schemas and descriptions are untrusted input.
- MCP tool results are untrusted output.

v0 does not claim to sandbox local MCP servers.

This differs from some external clients that may provide their own sandboxing or permission systems. AgentCast docs and UI must describe AgentCast's actual local guarantees, not Claude Code's or another host's guarantees.

## Executable Code

Rules:

- never turn registry command strings into shell execution.
- store command and args as structured fields.
- show command, args, cwd, and env requirements in install-plan previews.
- require confirmation before applying install plans.
- capture spawn failures as structured errors.
- treat MCP tool annotations as untrusted hints unless the server itself is trusted.

## Environment And Secrets

Rules:

- secrets must not be written to ordinary config by default.
- env forwarding must be explicit.
- install plans list required env vars without storing secret values.
- CLI and API output must redact common secret fields.
- tool arguments and results must not be logged by default when they may contain secrets.

## Confirmation Gates

Require confirmation for:

- destructive launcher actions.
- unknown-risk actions when policy is strict.
- actions whose safety classification relies only on untrusted upstream annotations.
- install-plan apply.
- config writes.
- process execution that was not already configured.

## Audit Log Expectations

v0 should be able to record:

- action id.
- server id.
- tool name.
- timestamp.
- outcome.
- normalized error kind.
- whether confirmation was required.

Do not store full arguments/results in audit logs by default.

## Out Of Scope For v0

v0 does not provide:

- filesystem sandboxing.
- container isolation.
- network egress controls.
- per-tool capability jails.
- malware scanning.
- registry trust scores.

Those can be added later, but docs and UI must not imply they exist.

For future Streamable HTTP support, local servers must follow upstream Origin validation, localhost binding, and authentication guidance before AgentCast exposes non-stdio defaults.

## Upstream References Checked

- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`: human-in-the-loop recommendation and untrusted tool annotations.
- `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`: `ToolAnnotations` are hints and must not drive decisions for untrusted servers.
- `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`: Streamable HTTP Origin/auth/localhost security guidance.
- `docs/references/mcp/docs/markdown/0120-modelcontextprotocol-io-docs-tutorials-security-security-best-practices.md`: MCP security best-practice context.
- `docs/references/claude-code/markdown/0096-code-claude-com-docs-en-security.md`: external Claude Code sandbox/permission comparison.
