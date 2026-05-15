---
title: "MCP Launcher MVP Contract"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: true
upstream_refs:
  - "docs/references/jmagar/jmagar-lab.xml"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# MCP Launcher MVP Contract

This document constrains the first implementation slice. The rest of `docs/` describes the intended end shape for AgentCast; this file decides what ships first.

## Product Statement

AgentCast v0 is an MCP-first launcher/runtime.

Operational target: one binary, local-first/self-hosted behavior, local SQLite store when persistence is needed, one local port only when serving API/MCP surfaces, and no required external hosted dependencies for already configured local workflows.

The first product loop is:

1. discover configured MCP servers.
2. normalize their tools into launcher actions.
3. search or list actions quickly.
4. invoke a selected tool without an LLM.
5. display structured results.
6. install or configure more MCP servers from the official MCP Registry.

The future shape still includes ACP sessions, loadouts, marketplace-compatible artifacts, remote execution, and agent operating-plane workflows. Those contracts remain valid, but they must not pull v0 away from the MCP launcher loop.

The CLI/API/MCP proof should lean on the strongest FastMCP and mcporter ideas: discover configured servers, list tools/resources/prompts, call tools directly, read resources directly, and generate deterministic interfaces from schemas.

The official MCP Registry is still a preview upstream surface. v0 registry code must cache locally, track freshness/provenance/status, tolerate pagination and data resets, and avoid assuming upstream uptime or data durability.

## Bootstrap Source

The MCP launcher MVP should reuse proven Lab implementation patterns where they are generic enough for AgentCast. Use `../lab` and `docs/references/jmagar/jmagar-lab.xml` as source material, following `docs/plans/extract-crates/README.md`.

Do not port Lab-specific homelab service behavior into the MVP.

## In Scope For v0

- CLI-first proof of the runtime model.
- Configured MCP server loading from AgentCast config and imported MCP JSON config.
- Existing MCP client config discovery for Claude Code, Codex, Gemini CLI, and project MCP config files.
- Deduplicated MCP server inventory.
- Local stdio MCP server spawning.
- MCP tool discovery.
- MCP resource discovery and read support.
- MCP prompt discovery when supported.
- MCP tool invocation.
- Normalized `LauncherAction` catalog.
- Search/list output suitable for a command palette later.
- Structured invocation results.
- Complete generic gateway behavior for v0: catalog merging, collision handling, exposure policy, routing, search handoff, protected public MCP routes, and upstream OAuth lifecycle.
- Protected public MCP route management and OAuth protected-resource metadata for gateway-served MCP routes.
- Upstream OAuth probe, authorization, callback completion, status, refresh, and credential clearing for OAuth-backed MCP upstreams.
- MCP Registry aggregator support: fetch, paginate, cache, index, normalize, track upstream status, and expose official-MCP-registry-compatible metadata locally where practical.
- Official MCP Registry install-plan generation from the local aggregator cache/index.
- Thin API/MCP surfaces only after the shared runtime path exists.

## Out Of Scope For v0

- Full ACP session hosting.
- Multi-agent orchestration.
- `agent-compose.yaml` execution.
- loadout creation, stash, fleet, AgentHub, or remote device deployment.
- Claude/Codex marketplace installation beyond preserving compatible future contracts.
- Desktop global hotkey UI.
- Docker, Kubernetes, or SSH execution targets.
- LLM-driven orchestration as the default command path.
- Custom frontend extension APIs or arbitrary extension-provided React/rendering code.

## Build Order

1. Protocol and config models: server ids, tool ids, launcher actions, invocations, results, server config.
2. `agent-config`: load AgentCast config, import MCP JSON config, discover existing client configs, dedupe servers.
3. `agent-mcp`: connect, list tools/resources/prompts, call tools, read resources, normalize MCP errors.
4. `agent-runtime`: start configured servers, discover tools/resources/prompts, cache catalog, invoke actions.
5. `agent-gateway` + `agent-auth`: merge catalogs, resolve collisions, apply aliases, route action to server/tool, manage protected public MCP routes, and orchestrate upstream OAuth lifecycle over auth/store primitives.
6. `agent-cli`: list servers/resources/tools/prompts, read resources, invoke an action, manage protected routes, run upstream OAuth probe/authorize/status/clear flows, render JSON/human output.
7. `agent-registry`: act as a local MCP Registry aggregator/subregistry: fetch official registry pages, cache/index normalized records, track freshness/status, and expose OpenAPI-compatible metadata where practical.
8. `agent-marketplace`: produce previewable MCP server install plans from registry/local metadata.
9. `agent-api` and MCP gateway endpoint: expose the same runtime behavior without adding product logic.

## Transport Order

1. `local-stdio`: first required MCP transport.
2. `streamable-http`: next standard MCP transport once local stdio is stable.
3. `ssh-stdio`: AgentCast execution topology for running stdio remotely, not a separate MCP transport; reserved architecture, not v0 unless explicitly promoted.
4. Docker, Kubernetes, and AgentCast node process: future execution targets.

## Runtime Boundary

CLI, API, MCP gateway, and future UI must all call the same runtime path:

```txt
surface -> runtime/gateway -> agent-mcp -> configured MCP server
```

No surface may implement its own MCP discovery, action routing, install policy, or result normalization.

## Acceptance Criteria

v0 is complete when a user can:

1. run a one-line bash install script.
2. configure nothing manually.
3. run `agentcast servers list` or `ac servers list` and see currently configured MCP servers from Claude Code, Codex, Gemini CLI, project MCP configs, and AgentCast config, deduped where those config sources are present and supported.
4. run `agentcast resources list` or `ac resources list` and see resources grouped by server.
5. run `agentcast resources read <server-id> <uri>` or `ac resources read <server-id> <uri>` and read a resource.
6. run `agentcast tools list` or `ac tools list` and see tools grouped by server.
7. run `agentcast prompts list` or `ac prompts list` and see prompts grouped by server when supported.
8. run `agentcast actions list` or `ac actions list` and see normalized launcher actions.
9. run `agentcast call <action-id> [args]` or `ac call <action-id> [args]` and invoke a discovered MCP tool deterministically through the normalized action path.
10. perform the same server/tool/resource/prompt/action operations through AgentCast MCP tools and API endpoints.
11. configure and validate protected public MCP routes with OAuth protected-resource metadata.
12. run upstream OAuth probe/authorize/status/refresh/clear flows for OAuth-backed MCP upstreams without hardcoding provider secrets.
13. add a server from raw MCP JSON config and have it work.
14. sync/cache official MCP Registry entries through AgentCast's local aggregator/index.
15. search registry entries from the local cache/index with provenance and freshness metadata.
16. preview an install/config plan for an MCP server.
17. see structured success or highly informative error output.
18. run tests that cover discovery, dedupe, resource listing/reading, tool listing, prompt listing, collision handling, invocation, protected route matching/metadata, upstream OAuth lifecycle fixtures, registry aggregation/cache/index behavior, and install-plan generation without network by default.

Setup performance criteria:

- not installed to fully setup should usually complete in under 2 minutes.
- fully cold start must complete in 5 minutes or less.
- setup must not require cloud account creation, hosted control-plane provisioning, or manual config editing for already configured MCP clients.

## Future Contracts

Existing docs for ACP, marketplace, gateway, loadouts, remote transports, and AgentHub describe the intended end-state architecture. They should be preserved and refined, not deleted. If a future-shape contract conflicts with this MVP contract, the future contract should be labeled as post-v0 instead of removed.
