---
title: "Config Contract"
doc_type: "contract"
status: "active"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs:
  - "docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md"
  - "docs/references/claude-code/markdown/0112-code-claude-com-docs-en-settings.md"
  - "docs/references/fastmcp/docs/client.mdx"
  - "docs/references/fastmcp/docs/overview.mdx"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcporter/docs/config.md"
  - "docs/references/mcporter/docs/import.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Config Contract

This contract defines AgentCast configuration ownership for the MCP launcher MVP.

## Sources

Requirements:

- durable non-secret settings live in `config.toml`.
- `.env` is reserved for secrets, endpoint URLs, tokens, API keys, and runtime process environment values.
- generated install flows must not create or require a large `.env` full of non-secret settings.
- config readers may load `.env` values only when a config entry explicitly references the key or a runtime adapter requires that process environment value.
- CLI flags may override config for a single command, but must not silently persist unless the command is explicitly a config mutation command.

## `config.toml` Ownership

`config.toml` owns:

- MCP upstream IDs, transport type, command, args, cwd, and enabled state.
- action aliases, categories, risk labels, and visibility overrides.
- registry provenance and install metadata that does not contain secrets.
- cache, timeout, retry, search, logging, and UI preference settings.
- store paths and non-secret runtime defaults.

## MCP JSON Import

AgentCast must be able to import existing MCP JSON config entries without requiring users to rewrite them by hand.

Requirements:

- imported server IDs are stable and deduped.
- unsupported source fields are preserved when practical or reported explicitly.
- imported non-secret settings become `config.toml`.
- imported secret/runtime env references remain `.env` key references or unresolved requirements.
- import diagnostics explain what was kept, changed, skipped, or blocked.

## `.env` Ownership

`.env` owns only:

- secrets.
- endpoint URLs that are environment-specific.
- tokens and API keys.
- runtime process environment values that child tools must receive.

Requirements:

- `.env` values must never be copied into install-plan previews, logs, audit records, or generated docs.
- plans and config may list required `.env` key names.
- missing required `.env` keys fail validation unless the caller explicitly allows deferred runtime binding.
- redaction uses the canonical literal from `docs/contracts/errors.md`.

## MCP Upstreams

Every MCP upstream in config must have:

- stable server ID.
- transport.
- launch/connect details for that transport.
- explicit environment allowlist for child process env injection.

For stdio upstreams, inherited parent process environment is denied by default. Child env is assembled from explicit config values and approved `.env` keys only.

This deny-by-default stdio environment policy is an AgentCast local security decision. Upstream tools such as Claude Code, FastMCP, and mcporter support env injection and discovery, but do not require this exact separation model.

## Mutations

Requirements:

- config mutations preserve unrelated sections and comments when practical.
- writes are atomic from the user perspective.
- conflicts are reported before writing unless the caller requested an explicit overwrite mode.
- secret values are never written to `config.toml`.

## Acceptance Tests

Implementations must test:

- non-secret MCP upstream settings round-trip through `config.toml`.
- secret references do not serialize secret values.
- missing required `.env` keys produce normalized config errors.
- parent env is not inherited for stdio upstreams unless explicitly allowed.
- install-plan apply writes durable settings to `config.toml`, not `.env`.

## Upstream References

- `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`
- `docs/references/claude-code/markdown/0112-code-claude-com-docs-en-settings.md`
- `docs/references/fastmcp/docs/overview.mdx`
- `docs/references/fastmcp/docs/client.mdx`
- `docs/references/mcporter/docs/import.md`
- `docs/references/mcporter/docs/config.md`
- `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`
