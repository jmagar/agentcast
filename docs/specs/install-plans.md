---
title: "Install Plan Spec"
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

# Install Plan Spec

## Owning Crates

- `agent-registry`: source metadata and normalized candidates.
- `agent-marketplace`: plan generation and conflict detection.
- `agent-config`: config mutation.
- `agent-runtime`: apply and verify.
- `agent-cli`/`agent-api`: preview/apply surfaces.

## Data Flow

```txt
registry entry -> install candidate -> install plan preview -> apply request -> config mutation -> runtime verification
```

## Plan Generation

Plan generation should:

1. select target `local-agent-runtime`.
2. derive server id.
3. build config table path `mcp.upstreams.<server-id>`.
4. compute the structured server config.
5. compare against existing config.
6. emit required `.env` key references for secrets/runtime URLs only.
7. emit conflicts and verification steps.

## Plan Apply

Apply should:

1. require confirmation.
2. reject unresolved conflicts.
3. write config through `agent-config`.
4. write `.env` only for user-supplied secret/runtime values.
5. start or reload the MCP server through runtime.
6. initialize and list tools.
7. return applied changes plus verification result.

## Config Versus Env

- `config.toml` stores server ids, transport, command, args, cwd, enabled flags, aliases, categories, risk overrides, registry provenance, install metadata, and env var names.
- `.env` stores secret values, endpoint URLs, access tokens, API keys, and process env values required at runtime.
- generated `.env` files must stay small and value-focused; package metadata, descriptions, schemas, defaults, and install history do not belong there.

## Env Policy

Plan generation uses the contract policy names:

- `fail_missing`: default; unresolved required `.env` keys block apply.
- `allow_missing`: config may reference missing `.env` keys, but verification is reported as blocked until values exist.

## Verification

Run:

```bash
cargo test -p agent-registry
cargo test -p agent-marketplace
cargo test -p agent-config config_mutation
```

## Upstream References

- `docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md`
- `docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md`
- `docs/references/fastmcp/docs/install-mcp.mdx`
- `docs/references/mcporter/docs/config.md`
- `docs/references/mcporter/docs/install.md`
- `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`
