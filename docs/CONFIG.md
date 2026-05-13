---
title: "Configuration Contract"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/claude-code/markdown/0100-code-claude-com-docs-en-env-vars.md"
  - "docs/references/claude-code/markdown/0112-code-claude-com-docs-en-settings.md"
  - "docs/references/fastmcp/docs/install-mcp.mdx"
  - "docs/references/fastmcp/docs/running.mdx"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Configuration Contract

`agent-config` owns configuration loading and validation.

For v0, config must be sufficient to load MCP servers for the launcher runtime.

## Config Sources

Initial precedence:

1. CLI flags.
2. process environment.
3. project config.
4. user config.
5. defaults.

## Config Files

Suggested locations:

```txt
./agentcast.toml
~/.agentcast/config.toml
~/.config/agentcast/config.toml
```

## MVP MCP Server Config

Initial config should support local stdio MCP servers first:

```toml
[mcp.servers.filesystem]
transport = "local-stdio"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
env = {}
enabled = true
```

`local-stdio` is AgentCast's internal transport label. The upstream MCP transport name is `stdio`, where the client launches the server as a subprocess and exchanges newline-delimited JSON-RPC over stdin/stdout.

Streamable HTTP can be added after local stdio is stable:

```toml
[mcp.servers.remote]
transport = "streamable-http"
url = "https://example.com/mcp"
enabled = true
```

`streamable-http` maps to MCP Streamable HTTP, not the deprecated HTTP+SSE transport. SSH, Docker, Kubernetes, and AgentCast node targets are future AgentCast deployment adapters; they are not separate standard MCP transports.

## Secrets

Secrets must not be stored in ordinary config by default.

Allowed sources:

- environment variables.
- OS keychain later.
- explicit local secret store later.

## Config Must Be Explicit

No crate except `agent-config` should discover config files. Runtime receives resolved config structs.

## Upstream References Checked

- `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`: upstream stdio and Streamable HTTP transport semantics.
- `docs/references/claude-code/markdown/0112-code-claude-com-docs-en-settings.md`: external client settings/config precedence comparison.
- `docs/references/claude-code/markdown/0100-code-claude-com-docs-en-env-vars.md`: MCP and secret-related environment behavior in Claude Code.
- `docs/references/fastmcp/docs/running.mdx`: FastMCP stdio/default and HTTP run modes.
- `docs/references/fastmcp/docs/install-mcp.mdx`: MCP config JSON generation and env handling.
