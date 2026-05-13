---
title: "Security Contract"
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
  - "docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0120-modelcontextprotocol-io-docs-tutorials-security-security-best-practices.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Security Contract

This contract defines mandatory v0 safety behavior.

## Local Execution

Requirements:

- local stdio MCP servers are treated as trusted executable code.
- install-plan preview must show command, args, cwd, and env requirements.
- runtime must not execute registry-provided shell strings.
- command execution must use structured command and args.

## Secrets

Requirements:

- secret values must not be written to ordinary config by default.
- `.env` is only for secrets, endpoint URLs, tokens, API keys, and runtime process env values.
- non-secret configuration must live in `config.toml`.
- secret-looking fields must be redacted from logs and error details.
- env var names may be displayed; secret values may not.
- audit records must not store full arguments or results by default.

## Confirmation

Confirmation is required for:

- destructive launcher actions.
- execute-risk launcher actions.
- unknown-risk launcher actions unless policy explicitly allows.
- install-plan apply.
- config writes.

## No Sandbox Claim

v0 must not claim filesystem, process, or network sandboxing for local stdio MCP servers.

The no-sandbox statement is intentional: upstream docs describe local stdio servers as local processes and warn users to trust MCP servers, but AgentCast v0 does not add an isolation layer.

## Acceptance Tests

Implementations must test:

- install preview displays execution details.
- high-risk actions require confirmation.
- denied confirmation does not invoke the tool.
- redaction removes common secret fields from error/log detail fixtures.

## Upstream References

- `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`
- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`
- `docs/references/mcp/docs/markdown/0120-modelcontextprotocol-io-docs-tutorials-security-security-best-practices.md`
- `docs/references/claude-code/markdown/0083-code-claude-com-docs-en-permissions.md`
- `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`
- `docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md`
