---
title: "Config Spec"
doc_type: "spec"
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
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Config Spec

This spec describes the v0 implementation shape for the config contract.

## Owning Crates

- `agent-config`: load, parse, validate, merge, and write `config.toml`.
- `agent-runtime`: consume validated runtime settings without reinterpreting source files.
- `agent-marketplace`: produce config mutations from install plans.
- `agent-cli`: expose config-affecting commands and validation errors.

## Module Shape

Initial `agent-config` modules:

- `lib.rs`: public exports.
- `error.rs`: normalized config error type and kind mapping.
- `model.rs`: top-level config structs.
- `mcp.rs`: MCP upstream config structs and validation.
- `env.rs`: `.env` key lookup and allowlist resolution.
- `write.rs`: atomic config mutation helpers.
- `paths.rs`: project and user config path resolution.
- `import.rs`: MCP JSON config import and source discovery.

## Load Algorithm

1. resolve config paths.
2. parse `config.toml` if present.
3. apply built-in defaults for omitted non-secret settings.
4. load `.env` key-value pairs into a restricted lookup table.
5. validate that required secret/runtime keys are present unless deferred env binding is enabled.
6. return a typed config object plus source diagnostics.

Environment variables must not override arbitrary config keys. The only env reads allowed in v0 are explicit `.env` key references, runtime process values listed by an upstream, and well-known bootstrapping paths such as an explicit config path override.

## MCP Client Config Discovery

Discovery should inspect known Claude Code, Codex, Gemini CLI, project MCP config, and AgentCast config locations.

The importer should:

- parse source config files without mutating them.
- normalize discovered servers into `mcp.upstreams`.
- dedupe by stable source identity, command/args/cwd, and explicit server ID where available.
- report collisions instead of silently overwriting.
- preserve provenance for diagnostics and observability.

## TOML Shape

Example:

```toml
[mcp.upstreams.filesystem]
enabled = true
transport = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/home/jmagar/workspace"]
cwd = "/home/jmagar/workspace"
timeout_ms = 30000

[mcp.upstreams.filesystem.env]
FILESYSTEM_ROOT = { source = "config", value = "/home/jmagar/workspace" }

[mcp.upstreams.github.env]
GITHUB_TOKEN = { source = "env", key = "GITHUB_TOKEN", required = true }

[launcher.aliases]
read-workspace-file = "mcp:filesystem:tool:read_file"
```

Allowed env entry sources:

- `config`: non-secret literal value in `config.toml`.
- `env`: lookup from `.env` or process env by key name.

Secret values must use `source = "env"`.

The `env` source is an AgentCast local abstraction. Resolution must prefer explicit `.env` bindings and only read process environment values when the resolved config entry explicitly allows that source.

## Validation Rules

Validation must reject:

- empty upstream IDs.
- duplicate aliases.
- secret-looking values in `config.toml` env entries.
- stdio upstreams with env passthrough set to all parent env.
- action aliases that point at malformed action IDs.
- `.env` key names that are empty or contain whitespace.

## Write Behavior

Install-plan apply and explicit config commands should:

- write non-secret durable settings to `config.toml`.
- write `.env` only when the user supplied a secret, endpoint URL, token, API key, or runtime process env value.
- preserve unrelated config sections.
- return a diff summary with secret values redacted.

## Verification

Add source-side test modules for:

- TOML parsing and defaulting.
- `.env` allowlist resolution.
- secret value rejection in `config.toml`.
- atomic mutation behavior.
- install-plan config mutation fixtures.

Run:

```bash
cargo test -p agent-config
```

## Upstream References

- `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`
- `docs/references/claude-code/markdown/0112-code-claude-com-docs-en-settings.md`
- `docs/references/fastmcp/docs/overview.mdx`
- `docs/references/fastmcp/docs/client.mdx`
- `docs/references/mcporter/docs/import.md`
- `docs/references/mcporter/docs/config.md`
- `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`
