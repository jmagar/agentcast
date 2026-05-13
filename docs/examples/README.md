---
title: "Examples"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md"
  - "docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md"
  - "docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md"
  - "docs/references/mcporter/docs/cli-reference.md"
  - "docs/references/mcporter/docs/livetests.md"
  - "docs/references/mcporter/docs/tool-calling.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Examples

These examples are concrete v0 shapes. They are illustrative until promoted into fixtures or tests.

## Upstream Reference Checks

- MCP lifecycle, stdio transport, tool discovery, and tool invocation semantics are checked against `docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md`, `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`, and `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`.
- MCP Registry examples are illustrative only; the registry is still documented as preview in `docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md` and package transport examples use `"type": "stdio"` in `docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md`.
- Claude Code local MCP config behavior is cross-checked against `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`.
- mcporter invocation examples and live validation expectations are cross-checked against `docs/references/mcporter/docs/tool-calling.md`, `docs/references/mcporter/docs/cli-reference.md`, and `docs/references/mcporter/docs/livetests.md`.

## Local Stdio MCP Config

```toml
[mcp.servers.filesystem]
transport = "stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
env = {}
enabled = true
```

## Registry Candidate

```json
{
  "id": "mcp-registry:filesystem",
  "source": "official-mcp-registry",
  "name": "Filesystem",
  "description": "Read and write files under configured roots.",
  "transport": "stdio",
  "package": {
    "manager": "npm",
    "name": "@modelcontextprotocol/server-filesystem"
  },
  "env_requirements": [],
  "raw": {}
}
```

## Generated Launcher Action

```json
{
  "id": "mcp:filesystem:tool:read_file",
  "source": {
    "kind": "mcp_server",
    "id": "filesystem"
  },
  "upstream_tool_name": "read_file",
  "title": "Read file",
  "description": "Read a file from an allowed filesystem root.",
  "category": "files",
  "risk": "read",
  "requires_confirmation": false,
  "input_schema": {
    "type": "object",
    "required": ["path"],
    "properties": {
      "path": {
        "type": "string",
        "description": "File path to read"
      }
    }
  },
  "search": {
    "terms": ["read file", "filesystem", "read_file"]
  },
  "metadata": {
    "transport": "stdio"
  }
}
```

## Tool Invocation Request

```json
{
  "action_id": "mcp:filesystem:tool:read_file",
  "arguments": {
    "path": "/tmp/example.txt"
  },
  "confirm": false
}
```

## Tool Invocation Result

```json
{
  "action_id": "mcp:filesystem:tool:read_file",
  "status": "ok",
  "content": [
    {
      "type": "text",
      "text": "hello\n"
    }
  ],
  "metadata": {
    "server_id": "filesystem",
    "tool_name": "read_file"
  }
}
```

## Install Plan Preview

```json
{
  "plan_id": "plan-filesystem-001",
  "candidate_id": "mcp-registry:filesystem",
  "target": "local-agent-runtime",
  "changes": [
    {
      "kind": "config_add",
      "path": "mcp.servers.filesystem",
      "value": {
        "transport": "stdio",
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
        "env": {},
        "enabled": true
      }
    }
  ],
  "conflicts": [],
  "risks": ["executes_local_command"],
  "verification": [
    "spawn",
    "initialize",
    "list_tools"
  ]
}
```
