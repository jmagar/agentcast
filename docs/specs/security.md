---
title: "Security Spec"
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

# Security Spec

This spec describes the initial implementation shape for the security contract.

## Owning Crates

- `agent-auth`: API auth policy and local bind checks.
- `agent-config`: `.env` and config separation.
- `agent-runtime`: child process environment assembly.
- `agent-gateway`: destructive action labeling and confirmation checks.
- `agent-observability`: redaction and audit event helpers.

## Env And Config

Implementation rules:

- `.env` stores only secrets, endpoint URLs, tokens, API keys, and runtime process environment values.
- `config.toml` stores all durable non-secret configuration.
- runtime env for stdio MCP servers is assembled from explicit config values and approved `.env` keys.
- parent process env is not inherited wholesale.
- serialized diagnostics use `[REDACTED]` for secret values.

## Confirmation Gates

Action metadata must include risk labels sufficient for CLI/API to decide whether confirmation is required.

For v0:

- read-only actions run without confirmation.
- destructive or external-write actions require confirmation.
- CLI interactive mode prompts before execution.
- CLI `--json` mode does not prompt; it returns a normalized confirmation-required error unless an explicit allow flag is supplied.
- API calls require an explicit confirmation field or equivalent auth-scoped decision.

Denied confirmations must not invoke the upstream MCP tool.

## Redaction

Redaction applies to:

- config diagnostics.
- install-plan previews.
- audit logs.
- API responses.
- CLI JSON output.
- panic-safe error formatting where practical.

Redaction must inspect nested JSON values and known secret key names. The canonical replacement literal is `[REDACTED]`.

## Local API Bind Policy

Startup checks:

- loopback bind may use explicit development mode without auth.
- non-loopback bind requires auth config.
- missing auth on non-loopback bind fails startup before any route is served.

No v0 implementation may document local stdio MCP execution as sandboxed unless a real isolation layer is added and contracted separately.

## Verification

Add source-side tests for:

- parent env deny-by-default.
- `.env` key allowlist behavior.
- confirmation-required short circuit before invocation.
- recursive redaction.
- non-loopback auth startup failure.

## Upstream References

- `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`
- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`
- `docs/references/mcp/docs/markdown/0120-modelcontextprotocol-io-docs-tutorials-security-security-best-practices.md`
- `docs/references/claude-code/markdown/0083-code-claude-com-docs-en-permissions.md`
- `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`
- `docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md`
