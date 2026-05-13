---
title: "Install Plans"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/fastmcp/docs/install-mcp.mdx"
  - "docs/references/mcp/docs/markdown/0038-modelcontextprotocol-io-registry-remote-servers.md"
  - "docs/references/mcp/docs/markdown/0046-modelcontextprotocol-io-registry-versioning.md"
  - "docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md"
  - "docs/references/mcp/docs/markdown/0172-modelcontextprotocol-io-registry-faq.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Install Plans

Install plans are the safe bridge between registry/marketplace discovery and filesystem/config mutation.

## Flow

v0 flow:

1. official MCP Registry entry or local registry fixture.
2. normalized install candidate.
3. install-plan preview.
4. user confirmation.
5. apply through runtime/config APIs.
6. verification through MCP runtime.

The official MCP Registry is currently preview. Install planning must tolerate breaking registry changes, stale local cache, and local fixtures during development.

## Install Candidate

An install candidate includes:

- `id`
- `source`
- `name`
- `description`
- `version`
- `transport`
- `package`
- `command`
- `args`
- `env_requirements`
- `config_template`
- `raw`

For MCP Registry candidates, package metadata comes from `server.json` `packages`; remote server metadata comes from `remotes`. AgentCast may normalize both, but must preserve the upstream source record.

## Install Plan

An install plan includes:

- plan id.
- candidate id.
- target.
- config changes.
- filesystem changes.
- environment requirements.
- conflicts.
- risks.
- verification steps.

Preview must not mutate files.

## Apply Rules

Apply must:

- require confirmation for writes.
- write through the same config mutation code used by manual config updates.
- preserve unrelated config.
- fail on conflicts unless the user selects an explicit resolution.
- record enough metadata to explain what changed.
- never execute registry-provided command text through a shell; store executable and arguments as structured fields.

## Verification

Verification may:

- check command availability.
- start the MCP server.
- initialize MCP.
- list tools.
- report discovered action count.

Verification failure does not roll back automatically in v0 unless rollback data is available. It must return a structured error and the applied change summary.

## Later Marketplace Scope

Post-v0 install plans may target:

- ACP providers.
- Claude Code plugins.
- Codex configs/plugins.
- AgentCast generated extensions.
- remote devices.

## Upstream References Checked

- `docs/references/mcp/docs/markdown/0172-modelcontextprotocol-io-registry-faq.md`: registry preview status and official API terminology.
- `docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md`: MCP Registry publish/install metadata flow.
- `docs/references/mcp/docs/markdown/0038-modelcontextprotocol-io-registry-remote-servers.md`: `remotes`, `packages`, Streamable HTTP, SSE, headers, and variables in `server.json`.
- `docs/references/mcp/docs/markdown/0046-modelcontextprotocol-io-registry-versioning.md`: immutable version metadata and package alignment.
- `docs/references/fastmcp/docs/install-mcp.mdx`: client install and generated MCP JSON patterns.
