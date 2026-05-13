---
title: "Marketplace Contract"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "end-state"
source_of_truth: false
upstream_refs:
  - "docs/references/claude-code/markdown/0093-code-claude-com-docs-en-plugins.md"
  - "docs/references/claude-code/markdown/0108-code-claude-com-docs-en-plugin-marketplaces.md"
  - "docs/references/mcp/docs/markdown/0038-modelcontextprotocol-io-registry-remote-servers.md"
  - "docs/references/mcp/docs/markdown/0172-modelcontextprotocol-io-registry-faq.md"
  - "docs/references/seed/chatgpt-windows-raycast-extensions.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Marketplace Contract

`agent-marketplace` owns install planning and marketplace component models.

This is an end-state contract. For v0, marketplace behavior is limited to previewable MCP server install plans fed by registry/local metadata.

## Scope

Marketplace covers:

- MCP servers.
- Claude Code plugins.
- Codex configs/plugins.
- ACP agents.
- generated AgentCast extensions.

Only MCP server config/install planning is required for the MVP.

Claude Code plugins, Codex configs/plugins, ACP agents, and generated extensions are target scope, not v0 commitments.

## Install Model

Marketplace does not immediately mutate files. It produces install plans.

Install plan stages:

1. resolve source entry.
2. inspect artifact/components.
3. select target runtime/client.
4. compute file/config changes.
5. present preview.
6. apply through runtime.
7. verify result.

## Install Targets

Targets should be explicit:

- local AgentCast runtime.
- AgentCast gateway upstream.
- Claude Code global config.
- Claude Code project config.
- Codex global config.
- Codex project config.
- future remote device.

For v0, target `local AgentCast runtime` first. External client targets are post-v0 unless explicitly promoted.

## Components

A marketplace artifact may contain:

- skills.
- subagents.
- slash commands.
- hooks.
- MCP server config.
- ACP provider config.
- scripts.
- docs.
- prompts.

Component metadata must include source path and target path before apply.

## Upstream References Checked

- `docs/references/mcp/docs/markdown/0172-modelcontextprotocol-io-registry-faq.md`: official MCP Registry terminology and preview status.
- `docs/references/mcp/docs/markdown/0038-modelcontextprotocol-io-registry-remote-servers.md`: registry `server.json` package/remote metadata.
- `docs/references/claude-code/markdown/0093-code-claude-com-docs-en-plugins.md`: Claude Code plugin concept.
- `docs/references/claude-code/markdown/0108-code-claude-com-docs-en-plugin-marketplaces.md`: Claude Code plugin marketplace model.
- `docs/references/seed/chatgpt-windows-raycast-extensions.md`: AgentCast product-origin decision to start with registry aggregation over a traditional marketplace backend.
