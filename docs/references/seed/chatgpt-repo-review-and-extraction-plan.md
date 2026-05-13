---
title: "AgentCast - Repo Review and Extraction Plan"
doc_type: "seed"
status: "historical"
owner: "refresh-docs"
audience:
  - "maintainers"
  - "contributors"
scope: "historical"
source_of_truth: false
upstream_refs: []
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "historical note reviewed against current local docs/references snapshot"
---

# AgentCast - Repo Review and Extraction Plan

**Source:** https://chatgpt.com/g/g-p-6a027d92fbc08191a17cbde6df8605a2-agentcast/c/6a033570-26f0-83ea-9c06-190ad2352433

**Historical scope note:** This is a seed transcript, not an authoritative current spec.
Protocol and registry claims should be checked against `docs/references/mcp/docs/`,
`docs/references/mcp/repos/`, `docs/references/acp/docs/`, and
`docs/references/acp/repos/`. The FastMCP/mcporter-shaped CLI/runtime guidance should be
checked against `docs/references/fastmcp/docs/`, `docs/references/fastmcp/repos/`, and
`docs/references/mcporter/docs/`.

---

Can you review jmagar/lab and then create a plan for extracting all of the crates

what the fuck - bro use the github connector to view jmagar/lab you fool

it wasnt a stale uploaded bundle you just used the straight up wrong fucking repo

Do we need the services shit at all for agentcast?

Should we even do this extraction in the lab repo, or should we just start a new repo and port over what we need?

is there a cargo command we're supposed to run to start a new rust workspace? like npx create-react-app or whatever

whats this do cargo init --vcs git

um arent we supposed to have more crates than that

Pasted text.txt
Document

Give me the full Cargo.toml for all 13 crates.

ok its building now - arent we missing some deps...? i didnt see any rmcp, acp related deps ?

so which of these crates is the dispatch layer ? where all the business lives ?

OK - I want you to start creating docs that can be used as contracts/specs to guide development

so isnt agentcast-mcp supposed to be a thin shim ?

cant you create the actual files?

ok are you able to read the other conversation in this project?

right - okay so the project obviously evolved significantly from what my initial idea was - a raycast like launcher for mcp servers.

now - we still want to support that - and that should still be what the initial product MVP is - and then we build up to the whole agent runtime / operating plane

---

## AgentCast Product Vision

That's the right framing: AgentCast MVP should be the Raycast-like MCP launcher, not the whole operating plane on day one. The architecture should support the bigger runtime, but the first product loop needs to stay narrow.

---

## Correct Product Framing

AgentCast is not starting as "agent operating plane."

AgentCast is starting as:

> A fast desktop launcher/control surface for discovering, installing, configuring, and invoking MCP servers.

Then it grows into:

> A unified MCP/ACP runtime, marketplace, gateway, and agent operating plane.

That matters because it changes what gets built first.

---

## MVP should be MCP-first

The MVP should answer:

> I want to hit a hotkey, search available MCP capabilities, run an action, see results, and install/configure new MCP servers without hand-editing JSON.

That is much sharper than "agent runtime."

### MVP product loop
1. Open launcher
2. → search commands/tools
3. → invoke MCP tool
4. → view result
5. → optionally pin/favorite/re-run

### MVP install loop
1. Open registry
2. → search MCP server
3. → inspect tools/env requirements
4. → install/configure
5. → generated launcher actions appear

---

## What this changes architecturally

### `agentcast-mcp` becomes immediately important

Still a thin shim, but MVP-critical.

It owns:
- connect to MCP server
- list tools
- call tool
- read resources/prompts later
- normalize MCP errors

### `agentcast-gateway` becomes less "fancy gateway" at first

For MVP, gateway means: local MCP server manager + merged launcher catalog

Not necessarily a hosted network gateway yet.

Initial responsibilities:
- configured MCP server list
- tool discovery
- tool indexing
- command/action search metadata
- name collision handling
- enabled/disabled tools
- route launcher invocation to correct MCP server

### `agentcast-runtime` starts smaller

Runtime does not need full agent sessions on day one.

Initial runtime responsibilities:
- spawn stdio MCP servers
- keep connections alive
- restart unhealthy servers
- cache discovered tools
- execute launcher actions
- stream execution result/status

Later runtime expands to ACP sessions, workspaces, event logs, agents, etc.

---

## Updated Crate Priority

You created 13 crates, which is fine. But priority should shift.

### Build first
- agentcast-core
- agentcast-config
- agentcast-protocol
- agentcast-mcp
- agentcast-runtime
- agentcast-registry
- agentcast-gateway
- agentcast-cli or desktop shell

### Build later
- agentcast-acp
- agentcast-marketplace
- agentcast-auth
- agentcast-api

They stay in the workspace, but they should not dominate MVP.

---

## MVP Crate Responsibilities

### `agentcast-protocol`

For MVP, define internal models like:
- `McpServerId`
- `McpToolId`
- `LauncherActionId`
- `LauncherAction`
- `ToolInvocation`
- `ToolInvocationResult`
- `ServerStatus`
- `InstallCandidate`

### `agentcast-mcp`

Thin wrapper:
- `connect_stdio_server`
- `connect_http_server`
- `list_tools`
- `call_tool`
- `shutdown_server`

### `agentcast-runtime`

Business execution:
- `start_configured_servers`
- `discover_all_tools`
- `build_launcher_catalog`
- `invoke_launcher_action`
- `track_server_status`
- `restart_server`

### `agentcast-gateway`

Catalog/routing policy:
- merge tool catalogs
- resolve collisions
- apply tool aliases
- apply enabled/disabled filters
- route action -> server/tool

### `agentcast-registry`

MCP Registry first:
- search MCP registry
- fetch server metadata
- extract package/env/install hints
- normalize into AgentCast install candidates

ACP Registry can come after.

---

## ADR: 0006 — MVP is a Raycast-like MCP launcher

AgentCast starts as a launcher/control surface for MCP servers.

The first product loop is:

1. discover configured MCP servers
2. index their tools as launcher actions
3. search actions quickly
4. invoke a tool
5. display structured results
6. install more MCP servers from registries

The broader agent runtime/operating plane remains the long-term direction, but it must not distort the MVP.

---

## Roadmap

### Phase 1 — Local MCP launcher
- config file for MCP servers
- start stdio MCP servers
- list tools
- build searchable local action catalog
- invoke tool
- show result
- CLI proof first if desktop shell is not ready

### Phase 2 — Registry install
- search official MCP Registry
- install server config
- handle env requirements
- refresh tool catalog
- basic validation

### Phase 3 — Desktop launcher UX
- global hotkey
- command palette
- server/tool search
- forms from JSON schema
- result view
- favorites/history

### Phase 4 — MCP gateway
- merged MCP endpoint
- tool filtering
- aliases
- auth
- remote HTTP MCP

### Phase 5 — ACP agents
- ACP provider registry
- start/connect agent
- create/list/resume sessions
- prompt stream UI

### Phase 6 — Agent operating plane
- session workspaces
- multi-agent dashboard
- logs/events
- install marketplace
- agent runtime management
- MCP-over-ACP / ACP-over-MCP bridges

---

## The key discipline

Do not build "agent OS" first.

Build this first: **Raycast for MCP tools.**

Then every later feature has a place to attach.
