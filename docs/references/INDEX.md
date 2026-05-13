---
title: "Reference Index"
doc_type: "reference-index"
status: "active"
owner: "refresh-docs"
audience:
  - "maintainers"
  - "contributors"
scope: "reference"
source_of_truth: true
upstream_refs: []
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Reference Index

This directory stores local reference material for protocol, SDK, CLI, and related project research.

## Layout

| Path | Contents | Source |
| --- | --- | --- |
| `acp/docs/` | Axon-crawled markdown docs and `manifest.jsonl` | `https://agentclientprotocol.com` |
| `acp/repos/` | Repomix XML packs for ACP repos | `agentclientprotocol/*` |
| `mcp/docs/` | Axon-crawled markdown docs and `manifest.jsonl` | `https://modelcontextprotocol.io` |
| `mcp/repos/` | Repomix XML packs for MCP repos | `modelcontextprotocol/*` |
| `claude-code/` | Axon-crawled markdown docs and `manifest.jsonl` | `https://code.claude.com/` |
| `fastmcp/docs/` | Sparse checkout of top-level FastMCP CLI `.mdx` docs | `PrefectHQ/fastmcp/docs/cli` |
| `fastmcp/repos/` | Repomix XML pack for the full FastMCP repo | `PrefectHQ/fastmcp` |
| `jmagar/` | Repomix XML packs for local/user-owned reference repos | `jmagar/*` |
| `mcporter/docs/` | Sparse checkout of mcporter docs | `openclaw/mcporter/docs` |
| `mcporter/repos/` | Repomix XML pack for mcporter | `openclaw/mcporter` |
| `seed/` | Raw origin transcripts and product-shaping notes | ChatGPT project conversations |
| `archive/changes-reports/` | Archived generated reference impact reports | Agentcast refresh workflow output |

## Reference Doc Copies

The ACP, MCP, and Claude Code docs are copied from Axon's host output under `~/.axon/output/domains/`. Each Axon-crawled directory contains:

- `manifest.jsonl` with crawl metadata.
- `markdown/` with one markdown file per crawled page.

FastMCP and mcporter docs are sparse checkouts, not Axon crawls. Their directories do not contain Axon manifests.

Current copied reference directories:

| Path | Files |
| --- | ---: |
| `acp/docs/` | 74 total: 73 markdown + 1 manifest |
| `mcp/docs/` | 189 total: 188 markdown + 1 manifest |
| `claude-code/` | 133 total: 132 markdown + 1 manifest |
| `fastmcp/docs/` | 7 sparse-checkout `.mdx` files |
| `mcporter/docs/` | 34 sparse-checkout files, including 32 markdown/docs files and 2 image assets |

## Repomix Packs

Repomix packs are consolidated XML snapshots intended for codebase-level reference and search.

### ACP Repos

- `acp/repos/agentclientprotocol-agent-client-protocol.xml`
- `acp/repos/agentclientprotocol-claude-agent-acp.xml`
- `acp/repos/agentclientprotocol-codex-acp.xml`
- `acp/repos/agentclientprotocol-registry.xml`
- `acp/repos/agentclientprotocol-rust-sdk.xml`
- `acp/repos/agentclientprotocol-typescript-sdk.xml`

### MCP Repos

- `mcp/repos/modelcontextprotocol-ext-auth.xml`
- `mcp/repos/modelcontextprotocol-modelcontextprotocol.xml`
- `mcp/repos/modelcontextprotocol-registry.xml`
- `mcp/repos/modelcontextprotocol-rust-sdk.xml`

### Other Packs

- `fastmcp/repos/prefecthq-fastmcp.xml`
- `jmagar/jmagar-aurora-design-system.xml`
- `jmagar/jmagar-lab.xml`
- `mcporter/repos/openclaw-mcporter.xml`
