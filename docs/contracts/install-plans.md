---
title: "Install Plan Contract"
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
  - "docs/references/fastmcp/docs/install-mcp.mdx"
  - "docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md"
  - "docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md"
  - "docs/references/mcporter/docs/config.md"
  - "docs/references/mcporter/docs/install.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Install Plan Contract

This contract defines safe install-plan behavior for v0 MCP server installation.

## Preview Rules

Preview must:

- not mutate files.
- include all proposed config changes.
- include required `.env` keys without secret values.
- include conflicts.
- include verification steps.
- include risk labels.

## Apply Rules

Apply must:

- require explicit confirmation.
- fail on unresolved conflicts.
- preserve unrelated config.
- write through AgentCast config mutation APIs.
- return applied changes and verification result.

## Conflict Rules

Conflicts must be explicit when:

- target server id already exists.
- target config path would overwrite a different value.
- required `.env` keys are missing and the install request does not explicitly select `deferred_env = "allow_missing"`.

## Env Policy

Install plans must keep durable configuration in `config.toml`. `.env` is reserved for secrets, endpoint URLs, tokens, and other values required by runtime processes that should not be committed to normal config.

Supported v0 env policies:

- `fail_missing`: default; apply fails when required `.env` keys are absent.
- `allow_missing`: write config that references env var names, but report verification as blocked until the user provides values.

Install plans must not turn registry metadata into a large generated `.env`; non-secret package metadata belongs in `config.toml`.

The `fail_missing` and `allow_missing` policy names are AgentCast local API terms. They map upstream registry and installer metadata into AgentCast config behavior without changing upstream MCP Registry, Claude Code, FastMCP, or mcporter formats.

## Acceptance Tests

Implementations must test:

- preview does not write files.
- apply writes expected config.
- conflicts block apply.
- unrelated config survives apply.
- verification failure is reported as `install_plan.verify_failed`.

## Upstream References

- `docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md`
- `docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md`
- `docs/references/fastmcp/docs/install-mcp.mdx`
- `docs/references/mcporter/docs/config.md`
- `docs/references/mcporter/docs/install.md`
- `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`
