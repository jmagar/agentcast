---
title: "AgentCast Principles"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
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

# AgentCast Principles

These principles capture the durable product and engineering philosophy from the seed conversations. They are not a replacement for contracts or specs; use them as the decision filter when implementation details are ambiguous.

## MVP Discipline

AgentCast starts as an MCP-first launcher/runtime, not a full agent operating plane on day one.

v0 should make this loop excellent:

1. discover configured MCP servers.
2. index their tools as launcher actions.
3. search actions quickly.
4. invoke a tool deterministically.
5. display structured results.
6. install/configure additional MCP servers from the official MCP Registry.

ACP, loadouts, stash, fleet, AgentHub, remote execution, and desktop polish should attach to this spine later instead of reshaping the MVP.

## Protocol-Native Product

AgentCast is not a Raycast clone with custom extensions. MCP servers are the capability/extension runtime, ACP agents are reasoning workers, and AgentCast is the host that makes both usable, searchable, configurable, and policy-controlled.

The product should be described as a protocol-native runtime, command center, or operating plane for tools and agents. Avoid reducing the product to a generic "AI launcher."

## Deterministic First

Every capability should be directly callable without an LLM.

Required interaction paths:

- deterministic CLI/API/MCP/runtime invocation.
- command-palette invocation.
- agentic orchestration on top of the same invocation path.

LLMs may choose and sequence tools later, but the underlying capabilities must stay inspectable, scriptable, testable, and callable without chat.

## Agents Are First-Class Users

Always assume an LLM agent will operate every surface: CLI, API, MCP, web, desktop, docs, logs, and config.

Agent-friendly means:

- stable machine-readable output.
- `--json` for CLI data commands.
- pagination for large collections.
- filtering for list/search commands.
- informative errors with structured details.
- discoverable commands, schemas, and examples.
- output that is concise by default but expandable when needed.
- observable runtime state instead of hidden side effects.

Humans and agents benefit from the same simplicity. Operator simplicity is agent operating simplicity.

## Simplicity Is Core IP

Simplicity is not just style. It is part of AgentCast's product advantage.

Prioritize systems that are:

- fast.
- simple.
- composable.
- reusable.
- portable.
- modular.
- organized.
- logical.
- reliable.
- professional.
- modern.

Every major design should be checked against operator simplicity and agent operating simplicity. If a design is hard for a human operator to understand, it will usually be harder for an LLM agent to operate safely.

Simplicity and speed apply everywhere:

- CLI commands should start fast, stream progress cleanly, and avoid noisy output.
- MCP tools should return bounded, structured responses quickly.
- API routes should paginate/filter instead of dumping large collections.
- web and desktop surfaces should feel immediate, searchable, and predictable.
- command palette interactions should be instant enough to stay in keyboard flow.
- setup should complete without hand-editing config for already configured MCP clients.

## Context And Token Discipline

Always be conscious of context and tokens.

AgentCast should reduce unnecessary context by:

- paginating large outputs.
- filtering before returning data.
- separating summaries from full details.
- keeping JSON stable and sparse by default.
- offering explicit expansion flags or detail endpoints.
- avoiding noisy logs on stdout.
- avoiding huge generated config/env files.

Do not force agents to parse unbounded text when a small structured response would work.

## Small Focused Modules

Modularity applies inside crates too. AgentCast should avoid large catch-all files.

Default code organization targets:

- source modules should target 250 non-test lines or less.
- source modules over 400 non-test lines need an explicit split/refactor note in the implementation plan or PR.
- source modules over 600 non-test lines require a documented exception.
- files over 800 total lines should be split unless they are generated, fixture data, or a clearly documented protocol/schema mirror.
- functions should target 60 lines or less.
- functions over 100 lines require a refactor or a documented exception.

Prefer focused files named after one responsibility over broad `utils`, `helpers`, or `manager` modules.


## Thin Surfaces

CLI, API, MCP, web, desktop, TUI, Raycast, scheduler, and future adapters are surfaces over the shared runtime.

Surface crates own:

- input parsing.
- output rendering.
- transport-specific protocol details.
- surface-specific confirmation UX.
- exit codes/status mapping.

Runtime/domain crates own:

- business decisions.
- config validation.
- catalog generation.
- routing.
- invocation.
- registry normalization.
- policy.
- persistence.

If behavior would be useful from more than one surface, it does not belong in CLI, HTTP route handlers, or UI code.

## Surface-Neutral Dispatch

Surface-neutral dispatch is the architectural spine. Model operations once and expose them through many surfaces.

The same internal operation should be reachable from:

- `agentcast` CLI.
- HTTP API.
- AgentCast MCP server.
- desktop/web UI.
- scheduler or loadout runner.
- ACP workflows.

Do not create parallel business paths per surface.

## Schema-Generated Extensions

AgentCast's differentiator is that users should not need to be extension developers.

If an MCP server exposes tools and schemas, AgentCast should be able to generate:

- launcher actions.
- CLI arguments.
- API DTOs.
- form fields.
- validation.
- search metadata.
- docs/help text.
- permission/risk prompts.
- result rendering fallbacks.

Bundled local MCPs may exist for demos or essential bootstrap flows, but built-in utility apps are not the product philosophy. The capability that matters is schema-driven extension generation from existing MCP servers.

Avoid custom extension frontend APIs as long as possible. Allowing arbitrary frontend code or custom React extension rendering harms consistency, security, determinism, and maintainability.

## Normalize The Ecosystem

Raw MCP and ACP ecosystems will be inconsistent. AgentCast should make them feel coherent.

Normalize:

- names.
- action IDs.
- aliases.
- categories.
- risk labels.
- auth/setup requirements.
- schema quirks.
- result shapes.
- registry metadata.
- errors.

This normalization layer is product value, not incidental glue.

Metadata normalization and the schema engine are core IP.

## Search And Indexing Are Infrastructure

Command-palette UX lives or dies on instant discovery.

Search/indexing should be treated as core infrastructure across:

- installed MCP tools.
- registry candidates.
- aliases.
- tags and categories.
- generated docs.
- loadouts and runs later.

Do not defer basic indexing until UI work. CLI and API should be able to prove discovery quality before desktop exists.

Prioritize local SQLite-backed cache/index storage, deterministic refresh behavior, and background sync/update hooks where appropriate. Instant discovery matters as much as protocol support.

## Official MCP Registry First

v0 registry support should start with the official MCP Registry and do it well.

The official MCP Registry is a preview upstream service. AgentCast should treat it as the source for v0 metadata, but it must not assume upstream uptime, stable data durability, or a final API until the upstream preview label is removed.

Other sources can become adapters later:

- Claude Code marketplaces/plugins/artifacts.
- Codex marketplaces/plugins/artifacts when a current upstream reference or AgentCast decision defines the supported format.
- Gemini extensions/artifacts when a current upstream reference or AgentCast decision defines the supported format.
- GitHub repos.
- local manifests.
- user stash.

Do not build a broad marketplace backend before the official MCP Registry flow is stable.

## FastMCP And mcporter Parity

The MVP should lean hard on the best ideas from FastMCP CLI and mcporter.

AgentCast should provide equivalent foundations:

- discover configured MCP servers from existing client configs and project files.
- list servers, tools, resources, and prompts.
- call MCP tools directly without chat or orchestration.
- read MCP resources directly.
- inspect schemas and metadata.
- generate deterministic interfaces from MCP schemas.

Full mcporter-style parity is part of the MVP foundation. The product can become broader later, but this layer needs to be complete and reliable early.

## Reuse Existing Operator Primitives

Prefer existing operator primitives over custom abstractions when they are already good.

Examples:

- `~/.ssh/config` over custom node inventory.
- git repos over custom marketplace hosting.
- Claude plugin/marketplace manifests over a new AgentCast plugin format.
- JSON Schema over a custom UI DSL.
- MCP Registry over custom server catalogs.
- OAuth/bearer auth over custom auth schemes.
- SQLite over distributed infrastructure for local state.
- standard MCP transports like stdio and streamable HTTP; avoid proprietary IPC unless there is a concrete need.
- static web exports over unnecessary frontend servers.

AgentCast should compose what operators already understand.

## Self-Hosted By Default

AgentCast must not depend on cloud services to function.

Core capabilities should be self-hosted/local-first:

- runtime.
- config.
- registry cache/index.
- search.
- install planning.
- MCP/ACP invocation.
- logs/audit/observability.

Cloud integrations may exist as optional MCP/ACP capabilities, but AgentCast itself must not require a hosted control plane, hosted registry service, hosted sync service, or cloud auth broker to operate.

The default stack should remain one binary, one local port when serving, local config, local SQLite storage, and no required external hosted dependencies. PostgreSQL can be a future optional backing store when SQLite is no longer enough.

## Transport Is Not Deployment Topology

Protocol semantics and deployment location are separate.

Protocol transport and deployment topology are different layers.

Stable MCP transports include:

- local stdio.
- streamable HTTP.

AgentCast execution topologies can wrap those transports, for example:

- SSH stdio.
- docker exec.
- kubectl exec.

ACP currently has stable stdio transport. ACP Streamable HTTP/WebSocket work is draft/proposal material until the upstream ACP protocol docs stabilize it.

End-state architecture may also support:

- websocket.
- future AgentCast node transports.

Stdio plus SSH is a first-class idea: it lets AgentCast run local-style protocols remotely without requiring HTTP servers, reverse proxies, custom auth, or TLS management.

Headline capability: run local agents and MCP servers anywhere you can SSH.

## Local-First And Portable

AgentCast should work locally first, then extend to remote devices through explicit transports.

The long-term direction is a portable operating plane for agent environments: run the right agent, with the right tools, in the right workspace, on the right target, under the right policy.

Remote/fleet/hub behavior should build on local runtime semantics instead of replacing them.

## Security Is Product Surface

Local stdio MCP servers are executable code. AgentCast's trust boundary is part of the product.

Security requirements should be visible in contracts and tests:

- explicit env allowlists.
- `.env` only for secrets, endpoint URLs, tokens/API keys, and runtime process env values.
- durable non-secret settings in `config.toml`.
- destructive action labeling.
- confirmation gates.
- audit logs.
- secret redaction.
- no full parent env inheritance by default.
- clear trust indicators for registry/install sources.

Security is not a later enterprise layer.

## Minimum Context, Maximum Efficiency

AgentCast should help agents run with only the context they need.

Loadouts/runs should eventually select:

- relevant agent.
- relevant MCP servers.
- relevant skills.
- relevant hooks.
- relevant subagents.
- relevant workspace.
- relevant policy.
- relevant prompt/context.

Avoid patterns that load every plugin, skill, tool, rule, and context block for every task.

## Compose Existing Artifacts

AgentCast should own orchestration, not artifact authority.

Use existing packaging ecosystems where possible:

- Claude Code plugin manifests.
- Claude-compatible marketplace entries.
- Codex artifacts only where supported by an explicit current upstream reference or an AgentCast decision; current Codex marketplace/plugin references in this repo are seed/Lab provenance, not an official upstream contract.
- MCP server configs.
- skills, commands, hooks, agents, and subagents from existing repos.

Long-term AgentCast-native files should describe composition and execution, not replace artifact packaging. A loadout is closer to an agentic `docker-compose.yml` than a new plugin format.

## Strict And Curated Artifact Modes

Artifact catalogs should support both:

- plugin-owned mode: the plugin manifest is authoritative.
- curated/generated mode: a marketplace or loadout entry can expose selected files/components when strict plugin ownership is not desired.

This keeps third-party plugins compatible while allowing generated/user-curated loadouts.

## Capability Hypervisor

AgentCast should feel like a capability hypervisor for MCP/ACP ecosystems.

One runtime should be able to expose a capability through:

- direct CLI call.
- API request.
- MCP tool.
- desktop/web command.
- scheduled run.
- ACP workflow.
- loadout recipe.

This framing keeps the architecture centered on capabilities instead of screens.

## Library First

Every reusable feature should start as library behavior. Binaries and apps are wrappers.

Prefer:

```txt
crates/agent-foo  -> reusable logic
apps/foo          -> deployable entrypoint/package
```

This keeps AgentCast modular and preserves reuse across future Rust projects.

## Composable Modules

Design reusable runtime capabilities as modules that can expose routes, CLI commands, MCP tools, migrations, and lifecycle hooks through one composition point.

The end-state shape should resemble:

```rust
trait Module {
    fn name(&self) -> &'static str;
    fn routes(&self) -> Router<AppState>;
    fn cli(&self) -> Command;
    fn mcp_tools(&self) -> Vec<ToolSpec>;
    fn migrations(&self) -> Vec<Migration>;
}
```

Use neutral framework patterns where they are reusable across future Rust apps. Product crates can remain AgentCast-branded, but the architecture should not trap general runtime patterns inside product-only surfaces.

## CLI/API/MCP Before UI

Validate the runtime model before building the desktop/web layer.

Early proof should come from:

- CLI commands.
- HTTP API.
- AgentCast MCP server.
- tests and fixtures.

The UI should visualize and operate proven runtime behavior, not invent behavior that only exists in the frontend.

## Everything Must Be Fast

Every surface should be fast enough for repeated human and agent operation.

From not installed to fully setup should be under two minutes on a normal developer machine.

Five minutes is the hard maximum for a fully cold start.

This includes install, first config discovery, local store initialization, and first successful `ac mcp list` for already configured MCP clients.

Avoid setup flows that require hand-editing config, cloud account creation, remote service provisioning, or long-running dependency bootstraps.

## Excellent Core UX

Build the foundation for:

- an insanely fast command palette.
- excellent MCP tool UX.
- excellent ACP session UX.

The first proof is CLI/API/MCP, but the runtime, schema, search, and metadata decisions should make those future UX surfaces feel first-class instead of bolted on.

## Pragmatic Stack

Rust owns runtime, process management, protocol adapters, security, config, indexing, registry normalization, persistence, and policy.

Web UI owns command surfaces, generated forms, tables, details, markdown, diffs, logs, settings, and agent session views.

Tauri plus React is the pragmatic default for desktop when desktop work begins. GPUI can be revisited later, but it is not the default path unless the project explicitly wants to take on GPUI maturity risk.

## Aurora Is Strategic

Aurora is a reusable operational UI foundation, not just styling.

It should inform future web/desktop surfaces for:

- command palette.
- generated forms.
- permission prompts.
- file tree.
- diff/log views.
- terminal-like output.
- tool-call rendering.
- agent session UI.

UI work should reuse this foundation where it fits instead of inventing one-off product screens.

## Extract From Lab, But Generalize

Lab already proves many hard runtime patterns. AgentCast should extract aggressively, but only across clean boundaries.

Extract patterns that survive without:

- Plex/Sonarr/Radarr/Unraid/Gotify assumptions.
- homelab service SDKs.
- appdata credential discovery.
- Lab-only node/fleet policy.
- Lab-specific UI copy.

The main extraction challenge is not missing features; it is finding the correct generalization boundaries.

## Current References During Planning

The MCP/ACP/FastMCP/mcporter/Codex/Claude ecosystem changes rapidly.

Major planning sessions must consult `docs/references/` before changing contracts, specs, or extraction plans. References should be refreshed daily when active development depends on fast-moving upstream behavior. Seed transcripts provide provenance; current contracts/specs and refreshed upstream references decide implementation behavior.

## Design For Containers Later

Container, Kubernetes, and stronger sandbox targets should be possible under the transport/execution-target model.

Do not build container execution first. It brings image management, workspace mounts, secret injection, Docker socket risk, cleanup, nested Docker, and network policy complexity.

Design the interfaces so these targets can attach later.
