---
title: "CLI"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/fastmcp/docs/install-mcp.mdx"
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

# CLI

The CLI is the first v0 surface. It must be thin over shared runtime and gateway behavior.

## Command Families

Initial commands:

```txt
ac mcp list
ac mcp resources <server-id> [uri]
ac resources list
ac tools list
ac prompts list
ac call <server-id> <tool-name>
agentcast servers list
agentcast servers get <server-id>
agentcast servers start <server-id>
agentcast servers stop <server-id>
agentcast resources list
agentcast resources read <server-id> <uri>
agentcast tools list
agentcast prompts list
agentcast actions list
agentcast actions search <query>
agentcast actions get <action-id>
agentcast call <action-id>
agentcast registry search <query>
agentcast install preview <candidate-id>
agentcast install apply <plan-id>
```

`ac` is the short operator/agent-friendly alias for `agentcast`. The long binary name remains valid for scripts that prefer explicit names.

Command names are AgentCast-local. They may wrap MCP operations such as `tools/list`, `resources/list`, `prompts/list`, `resources/read`, and `tools/call`, but the CLI should not expose raw JSON-RPC method names as the primary UX unless a debug/raw mode is added.

## Output Modes

Every command that returns data supports:

- human output by default.
- JSON output with `--json`.
- pagination with `--limit` and `--cursor` or an equivalent command-specific mechanism.
- filtering with `--filter`, command-specific flags, or query parameters where the data set can be large.

Rules:

- data goes to stdout.
- logs, progress, warnings, and errors go to stderr.
- `--json` output must be parseable JSON and must not mix progress text into stdout.
- machine-facing JSON shapes are stable once captured in `docs/contracts/cli.md`.
- list commands must not dump unbounded output by default.
- JSON list responses include pagination metadata when more results are available.
- human list output should summarize truncation and show the next command needed to continue.
- cursors are opaque; callers must not parse, store, or modify them across sessions.

## Invocation Input

`agentcast call` accepts:

```txt
agentcast call <action-id> --json-args '{"path":"/tmp"}'
agentcast call <action-id> --arg path=/tmp
agentcast call <action-id> --confirm
```

Rules:

- `--json-args` accepts one JSON object.
- `--arg key=value` is allowed only for simple scalar schema fields.
- unsupported schemas require `--json-args`.
- high-risk actions require `--confirm`.

## Agent-Friendly Rules

The CLI is a primary agent surface.

Requirements:

- all data commands support `--json`.
- all large list commands support pagination and filtering.
- errors are informative enough for an agent to decide the next action without scraping logs.
- examples in help output should be copy-pasteable.
- command names should be discoverable through `--help` and list/search commands.
- output should be concise by default and expandable with explicit flags.

## Exit Codes

Initial exit codes:

- `0`: success.
- `1`: general failure.
- `2`: usage or validation error.
- `3`: config error.
- `4`: runtime or process error.
- `5`: MCP protocol error.
- `6`: registry/install-plan error.
- `7`: confirmation required or denied.

## Thin Adapter Rule

The CLI may parse flags and render output. It must not:

- implement MCP discovery.
- spawn MCP servers directly.
- mutate config outside `agent-config`/runtime apply APIs.
- normalize registry entries itself.
- duplicate gateway collision policy.

## Upstream References Checked

- `docs/references/mcp/docs/markdown/0058-modelcontextprotocol-io-specification-2025-11-25-server-utilities-pagination.md`: opaque cursor pagination requirements.
- `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`: tool listing, invocation, schema, and annotation terminology.
- `docs/references/fastmcp/docs/overview.mdx`: FastMCP CLI patterns for `list`, `call`, `discover`, and `generate-cli`.
- `docs/references/fastmcp/docs/install-mcp.mdx`: generated MCP JSON and stdio command install patterns.
- `docs/references/mcporter/docs/cli-reference.md`: mcporter CLI patterns for list/call/resource/generate flows.
