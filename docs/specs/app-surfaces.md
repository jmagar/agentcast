---
title: "App Surfaces Spec"
doc_type: "spec"
status: "active"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs:
  - "docs/references/acp/docs/markdown/0073-agentclientprotocol-com-.md"
  - "docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md"
  - "docs/references/claude-code/markdown/0093-code-claude-com-docs-en-plugins.md"
  - "docs/references/claude-code/markdown/0110-code-claude-com-docs-en-skills.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# App Surfaces Spec

This spec defines the `apps/` directories. Reusable logic remains in `crates/`.

## Directory Boundaries

```txt
apps/cli
apps/mcp
apps/api
apps/web
apps/desktop
```

## `apps/cli`

Owns deployable CLI packaging and wrapper concerns.

It may eventually contain:

- binary crate wrapper.
- packaging scripts.
- shell completion assets.
- distribution metadata.

Reusable command parsing remains in `crates/agent-cli`.

## `apps/mcp`

Owns the deployable AgentCast MCP server entrypoint.

It may eventually contain:

- MCP server binary wrapper.
- stdio/HTTP launch configuration.
- smoke fixtures for MCP clients.

Reusable MCP protocol logic remains in `crates/agent-mcp` and gateway policy remains in `crates/agent-gateway`.

## `apps/api`

Owns deployable HTTP API app concerns.

It may eventually contain:

- API server binary wrapper.
- deployment config.
- OpenAPI publication assets.

Reusable Axum routes remain in `crates/agent-api`.

## `apps/web`

Owns the future frontend application.

It may eventually contain:

- web app source.
- static assets.
- design-system integration.
- Playwright tests.

Frontend code must consume API/runtime contracts, especially shared DTOs from `crates/agent-ui-contracts`, and must not implement MCP protocol behavior directly.

## `apps/desktop`

Owns the future desktop application surface.

It may eventually contain:

- Tauri or native desktop shell code.
- tray/menu integration.
- global hotkey wiring.
- deep link registration.
- updater packaging.
- desktop-specific smoke tests.

Reusable contracts remain in `crates/agent-ui-contracts`. Reusable native logic should move into a Rust crate only when it becomes substantial enough to test independently.

## Upstream References

- `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`
- `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`
- `docs/references/claude-code/markdown/0093-code-claude-com-docs-en-plugins.md`
- `docs/references/claude-code/markdown/0110-code-claude-com-docs-en-skills.md`
- `docs/references/acp/docs/markdown/0073-agentclientprotocol-com-.md`
