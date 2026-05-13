---
title: "CLI Contract"
doc_type: "contract"
status: "active"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs:
  - "docs/references/fastmcp/docs/client.mdx"
  - "docs/references/fastmcp/docs/generate-cli.mdx"
  - "docs/references/fastmcp/docs/overview.mdx"
  - "docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcporter/docs/cli-reference.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# CLI Contract

This contract defines command behavior and output rules for the AgentCast CLI.

## Output Rules

Requirements:

- command data goes to stdout.
- logs, progress, warnings, and errors go to stderr.
- `--json` stdout must be valid JSON.
- `--json` stdout must not contain progress text.
- errors under `--json` use the normalized error envelope from `docs/contracts/errors.md`.
- list/search commands support pagination.
- list/search commands support filtering when the result set can be large.
- list/search commands do not emit unbounded output by default.
- JSON list responses include `meta.next_cursor` or an equivalent stable continuation field when more results are available.

## Initial Commands

The v0 CLI must support:

- `ac mcp list`
- `ac mcp resources <server-id> [uri]`
- `ac resources list`
- `ac tools list`
- `ac prompts list`
- `ac call <server-id> <tool-name>`
- `agentcast servers list`
- `agentcast servers get <server-id>`
- `agentcast resources list`
- `agentcast resources read <server-id> <uri>`
- `agentcast tools list`
- `agentcast prompts list`
- `agentcast actions list`
- `agentcast actions search <query>`
- `agentcast actions get <action-id>`
- `agentcast call <action-id>`
- `agentcast registry search <query>`
- `agentcast install preview <candidate-id>`
- `agentcast install apply <plan-id>`

## Exit Codes

Requirements:

- `0`: success.
- `2`: usage or validation error.
- `3`: config error.
- `4`: runtime/process error.
- `5`: MCP protocol error.
- `6`: registry/install-plan error.
- `7`: confirmation required or denied.
- `1`: fallback for uncategorized failure.

## Acceptance Tests

Implementations must test:

- JSON mode produces parseable JSON.
- human mode stays on stdout.
- stderr receives errors/progress.
- list pagination does not drop or duplicate records across pages.
- list filtering applies before pagination.
- validation errors use exit code `2`.
- confirmation-required errors use exit code `7`.

## Upstream References

- `docs/references/fastmcp/docs/overview.mdx`
- `docs/references/fastmcp/docs/client.mdx`
- `docs/references/fastmcp/docs/generate-cli.mdx`
- `docs/references/mcporter/docs/cli-reference.md`
- `docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md`
- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`
