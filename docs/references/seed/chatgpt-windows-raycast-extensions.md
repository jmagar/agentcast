---
title: "AgentCast - Windows Raycast Extensions (Origin Conversation)"
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

# AgentCast - Windows Raycast Extensions (Origin Conversation)

**Source:** https://chatgpt.com/g/g-p-6a027d92fbc08191a17cbde6df8605a2-agentcast/c/6a021526-1cbc-83ea-8d7c-f1d3fb5a169c

**Historical scope note:** This is an origin conversation and includes product speculation,
third-party ecosystem observations, and future-facing architecture. Current MCP, ACP,
FastMCP, mcporter, and Claude Code plugin/marketplace claims should be checked against
`docs/references/mcp/`, `docs/references/acp/`, `docs/references/fastmcp/`,
`docs/references/mcporter/`, and `docs/references/claude-code/`. This repository does not
currently include upstream Raycast or Windows launcher reference snapshots under
`docs/references/`, so Raycast/Windows extension statements remain historical seed context.

---

## Turn 1

**You:** Can you search for popular windows raycast extensions

**ChatGPT:** If you're trying to recreate the experience on Windows, the ecosystem is currently split across Raycast (most "Raycast-like" for tinkerers), Microsoft PowerToys Command Palette / Run, the new Raycast Windows beta itself, and niche launchers like Keypirinha and Fluent Search.

### Most Popular / Useful Windows "Raycast-style" Extensions

**File Search via Everything** — This is basically mandatory.
- Flow Launcher + Everything integration
- PowerToys Run + Everything plugin

**Winget Package Installer** — Very popular in PowerToys Run.
- Search packages, install/update apps, avoid opening terminal/browser

**Browser / Web Search Plugins** — Common setups: Google, GitHub, StackOverflow, Reddit, YouTube, ChatGPT, Perplexity

**Clipboard History** — One of the most-used "Raycast" features overall.

**Window Walker / Window Switching** — Fuzzy-search open windows instead of Alt+Tab spam.

**Obsidian / Notes Search** — Search Obsidian vaults, create notes, append daily logs, fuzzy-search markdown.

**GitHub / Dev Plugins** — Open repos, search PRs/issues, clone repos, open local projects, copy SSH URLs.

**Calculator / Unit Conversion** — Built into nearly all of them.

**AI Integrations** — This category is exploding right now: ChatGPT launcher actions, Ollama local model prompts, Claude quick prompts, clipboard → AI rewrite, AI shell command generation.

### Current Ecosystem Reality

**Best overall for customization:** Flow Launcher — open-source, huge plugin ecosystem, closest to "Windows Alfred/Raycast," extensible, strong community.

**Best built-in Windows option:** Microsoft PowerToys — especially now that Command Palette is evolving aggressively.

**Most polished UX:** Raycast Windows beta — but ecosystem still immature, fewer extensions, less customizable, not fully parity with macOS yet.

### What I'd Actually Recommend For You (homelab, MCP, coding, automation, keyboard-heavy workflows)

Stack:
- Flow Launcher
- Everything integration
- Winget plugin
- GitHub plugin
- Obsidian plugin
- Clipboard manager
- custom AI actions/scripts

---

## Turn 2

**You:** Um I'm asking about raycast for windows not raycast like apps....

**ChatGPT:** Yeah — currently the Windows version of Raycast does support extensions now, but the ecosystem is still much smaller. The extensions people are actually using on Windows right now are mostly the cross-platform ones.

### Popular Raycast Extensions That Work on Windows

**GitHub** — Search repos, manage branches, issues/discussions, PR workflows. Mentioned repeatedly as one of the best Windows-compatible extensions.

**Obsidian** — Search vaults, create notes, quick capture, open markdown instantly. Works well because it doesn't depend on macOS native APIs.

**Home Assistant** — Control devices, trigger automations, view sensors, interact with entities. People specifically call this out as one of the coolest Windows-compatible integrations.

**Google Translate** — Simple but heavily used. Quick translation directly from launcher.

**TLDR Pages** — Popular terminal/dev extension. Search command cheatsheets quickly.

**Hacker News** — Common "news in launcher" extension.

**Skills** — Lets you search/manage AI agent "skills."

**Everything Search Integration** — Uses Voidtools Everything for ultra-fast file search on Windows. This should probably become a first-party integration eventually.

**Webhook Sender** — Fire HTTP webhooks, save presets, send JSON payloads. Good fit for homelab/admin workflows.

**Hermes Agent** — Local AI assistant integration. Lets Raycast talk to a local AI agent.

### Important Limitation Right Now

A lot of extensions are still tagged `macOS only` — especially ones requiring native macOS APIs, AppleScript, window management internals, Finder integrations, menu bar access.

Raycast explicitly says: "All extensions that do not require native code will work out of the box on both platforms."

### Current State of Raycast Windows

The Windows beta has accelerated hard recently. Recent additions: Script Commands, Quick Look, Translator, AI support, Clipboard improvements, Extension Store support.

Extensions I think you'd actually care about (based on your workflows): GitHub, Home Assistant, Obsidian, Webhook Sender, Everything Search, AI/agent extensions, any MCP-adjacent tooling that emerges.

---

## Turn 3

**You:** I want you to explore feasibility of creating something similar to raycast - but our "extensions" would be MCP servers (and ACP agents)

**ChatGPT:** I'll treat this as a product/architecture feasibility pass: what maps cleanly from Raycast extensions onto MCP/ACP, what the risks are, and what the MVP should be.

**Yes. But it should not be "Raycast clone + MCP." It should be a Feasible — and actually a strong idea — command palette where MCP servers are the extension runtime.**

### Core thesis

Raycast extensions are app-specific plugins written for Raycast. Your version would invert that:

Any MCP server becomes an installable command pack. That means GitHub, Linear, Notion, Home Assistant, Unraid, Tailscale, Plex, Obsidian, etc. could expose capabilities directly.

MCP already standardizes over JSON-RPC. That maps well to a launcher/command palette.

### Mapping: Raycast → MCP launcher

| Raycast concept | MCP equivalent | Fit |
|---|---|---|
| Extension | MCP server | Excellent |
| Command | MCP tool | Excellent |
| Preferences | Server config / auth / env | Good |
| AI Extension | Tool callable by model | Excellent |
| Quicklinks | Static tools / templates | Good |
| Forms | Tool input schema → generated form | Excellent |
| Lists / detail views | Tool result renderer | Medium |
| Clipboard/snippets | Built-in local server | Excellent |
| File search | Built-in local server | Excellent |
| Window management | Native local server | Good, OS-specific |

### Why this is viable now

MCP has crossed the "real ecosystem" threshold.

### The product shape

**Labby Command Center** — A Windows-first Raycast-style launcher where:
- Extensions are MCP servers
- Commands are MCP tools
- Forms are generated from JSON schemas
- AI can call installed tools
- Local automation is handled by trusted local MCP servers
- Remote integrations use OAuth-backed MCP servers
- Homelab services become first-class command packs

### MVP architecture

```
Keyboard Shortcut
→ Command Palette UI
→ Local Host App
→ MCP Client Manager
→ Installed MCP Servers
  ├─ stdio local servers
  ├─ Streamable HTTP remote servers
  ├─ built-in Windows server
  ├─ built-in clipboard/snippets server
  └─ built-in homelab/service servers
```

### Critical design decision

Do NOT require an LLM for every command. There should be two paths:

**1. Deterministic command mode** — The launcher directly calls MCP tools.

**2. AI orchestration mode** — Then the model can inspect tool descriptions, choose the right MCP tools, and execute with confirmation.

### Biggest feasibility risks

1. **MCP tools are not UI extensions** — You need a generic renderer: table, list, markdown, JSON tree, form, confirmation dialog, progress view, error view.
2. **Tool naming will be messy** — You need a normalization layer.
3. **Security is the product** — Local stdio MCP servers inherit local machine access, so the host app becomes the trust boundary.
4. **MCP remote transport is still evolving** — Streamable HTTP is production-ready but still has gaps.

### MVP I'd build first

**Phase 1: Local Windows command palette** — global hotkey, app/file search, clipboard history, snippets, calculator, web search, command history, settings UI, MCP server install/config screen.

**Phase 2: MCP extension runtime** — stdio MCP servers, remote MCP servers, tool discovery, generated forms from input schemas, tool result rendering, OAuth/token config, per-tool permissions, audit log.

**Phase 3: AI layer** — natural-language command routing, tool selection, multi-step workflows, approval gates, saved workflows, "explain what you're about to do" mode.

### Killer differentiator

> An MCP-native operating console for local machines, cloud services, and homelabs.

The killer demo:
```
unraid status
→ Restart unhealthy containers, check logs, and send me a Gotify summary
```

That is not just Raycast-like. That is an agentic control plane.

---

## Turn 4

**You:** Yes and then definitely implement full ACP support.

**ChatGPT:** ACP changes the shape meaningfully. Make ACP a separate plane, not an "extension type."

### Clean mental model

| Layer | Protocol | Purpose |
|---|---|---|
| Tools / integrations | MCP | Connect to services, files, APIs, homelab systems |
| Coding agents | ACP | Connect Claude Code, Codex CLI, Gemini CLI, Cline, etc. |
| App shell | Your launcher | UI, permissions, routing, audit logs, install UX |

### Why ACP belongs here

MCP makes your app an **extension host**. ACP makes your app an **agent host**.

### Architecture adjustment

```
Desktop Shell
→ Command Palette / Dashboard
→ Orchestrator Core
  ├─ MCP Manager
  │  ├─ server registry
  │  ├─ permissions
  │  ├─ tool schemas
  │  └─ tool execution
  ├─ ACP Manager
  │  ├─ agent registry
  │  ├─ session lifecycle
  │  ├─ workspace context
  │  ├─ message streaming
  │  └─ diff / approval UI
  └─ Policy Engine
     ├─ secrets
     ├─ sandboxing
     ├─ audit log
     └─ confirmation gates
```

### Important boundary

MCP servers should be treated as **capabilities**. ACP agents should be treated as **reasoning workers**. Keep them separate.

### Killer workflow

```
Ask Claude to update my Unraid Gotify script.
Use repo context, check current files, and produce a PR.
```

- ACP starts Claude/Codex session
- Launcher passes workspace context
- Agent reads repo
- Agent can request approved MCP tools
- User sees diffs
- User approves shell/git actions
- Session log is saved automatically

### MVP priority

1. Command palette + local MCP hosting
2. MCP server registry/install/config
3. Generated forms/results for MCP tools
4. ACP client support for one agent
5. Session UI: logs, diffs, approvals
6. Multi-agent support
7. Shared session-log workspace
8. Marketplace for MCP servers + ACP agents

Full ACP support is the right call. It turns the product from "Raycast but protocol-based" into "The desktop control plane for tools, agents, and automation."

---

## Turn 5

**You:** And what language would you suggest? I was thinking rust + gpui?

**ChatGPT:** Yes: Rust is the right backend language. I'd be cautious about GPUI for the main app today.

### My recommendation — Best practical stack

```
Rust core + Tauri v2 + React/Svelte frontend
```

Use Rust for: MCP client/host runtime, ACP client/session runtime, process spawning / stdio transport, HTTP/SSE/WebSocket transports, permissions, secrets, indexing, audit log, filesystem/native Windows integration.

Use web UI for: command palette, generated forms, tables/lists/detail panes, settings, marketplace, agent session views, diff viewers, markdown logs.

### Rust + GPUI?

Cool, but risky. GPUI is compelling — Rust-native, GPU-accelerated, built by the Zed team. But the ecosystem still looks immature compared with Tauri. API churn, weaker ecosystem, fewer production examples.

### Why Tauri fits this app better

This app needs a lot of UI that web tech already handles well: searchable command lists, dynamic forms from JSON Schema, markdown rendering, code blocks, diff viewers, logs, tables, plugin/marketplace cards, OAuth flows, settings panels, agent chat/session UI.

### Final call

Use: `Rust + Tauri v2 + React` or `Rust + Tauri v2 + SvelteKit`

Do NOT start with GPUI unless the project goal is partly "help mature GPUI."

---

## Turn 6

**You:** Rust + Tauri v2 + React — Any projects / code we can use as a reference?

**ChatGPT:** The important references are not "find Tauri apps" but specifically: command palette UX, extension runtime design, agent session UX, Rust desktop architecture, plugin/registry systems, keyboard-first interaction models.

### The most important references

**1. Zed** — Probably your single most valuable reference overall. Rust-heavy architecture, command palette, AI/agent integrations, extension system, workspace/session concepts, keyboard-first UX, async Rust desktop architecture.

**2. Raycast Extensions Repo** — Not for direct use, but gold for: command UX patterns, action naming, extension discoverability, list/detail/form flows, keyboard interaction design.

**3. Tauri v2** — IPC patterns, plugin system, updater, shell/process APIs, permissions model, multi-window handling.

**4. Awesome Tauri** — Reference harvesting list.

### Templates I would actually use

**dannysmith/tauri-template** — Tauri v2, React 19, production-grade structure, shadcn/ui, Zustand, TanStack Query, opinionated architecture.

**ultimate-tauri-template** — Good for cargo workspaces, split crates, maintainable architecture.

### MCP/ACP-specific references

- Model Context Protocol Spec — tools, resources, prompts, transport, cancellation, progress, capability negotiation
- Agent Client Protocol — agent/session architecture from day one

### Biggest recommendation

**Do NOT build:** marketplace, cloud sync, remote orchestration, GPUI-native UI first.

**Build:** insanely fast command palette, excellent MCP tool UX, excellent ACP session UX.

That is the actual moat.

---

## Turn 7

**You:** Don't really think we should study raycast extensions tbh - shouldn't we study the official mcp registry?

**ChatGPT:** Yes. That's the better reference for the runtime model. Your app is fundamentally: MCP host + ACP host + command palette, not a custom extension runtime.

### What you should actually study

**1. Official MCP Registry + ecosystem** — study server packaging patterns, tool schemas, auth flows, transport usage, capability negotiation, install/config UX problems, naming consistency.

The REALLY important thing: **Existing MCP server quality variance** — this will define your entire platform UX. Tool naming is inconsistent, schemas wildly variable, auth messy, output formatting inconsistent.

Your app's value becomes: **making chaotic MCP servers feel coherent and polished**.

### Your real product moat — Normalization layer

Raw MCP: `"name": "list_containers"`
Your UI: `Docker → List Containers` + Aliases: docker ps, containers + Category: Infrastructure + Risk: Read-only

### Architecture direction — Layer 1 (Runtime), Layer 2 (UX Engine), Layer 3 (Marketplace)

---

## Turn 8

**You:** Well another thing that we should thoroughly research is the fastmcp cli + mcporter - research both

**ChatGPT:** Yeah — these are extremely important references for your architecture. And honestly, this changes my recommendation a bit: Your launcher should probably expose its own deterministic CLI/runtime layer from day one.

### FastMCP — what it actually is

FastMCP started as a Python framework for building MCP servers. But it is evolving into: a full MCP runtime ecosystem, MCP client tooling, inspection tooling, CLI generation tooling, discovery tooling, transport abstraction layer.

Key commands:
```bash
fastmcp list
fastmcp call
fastmcp discover
fastmcp generate-cli
```

### The REALLY important FastMCP ideas

**1. `fastmcp discover`** — Scans Claude Desktop configs, Cursor configs, Gemini CLI configs, project mcp.json files and creates a unified MCP inventory. Your launcher should do exactly this.

**2. `fastmcp call`** — Protocol-native tool execution. No LLM. No chat UI. No orchestration. Just:
```bash
fastmcp call weather get_forecast city London
```

**3. `fastmcp generate-cli`** — FastMCP can inspect MCP tool schemas and auto-generate a standalone CLI app from them. MCP schema → deterministic interface.

### mcporter — why it matters

FastMCP = build servers. mcporter = directly consume/use servers. MCP should feel like infrastructure tooling: discoverable, callable, inspectable, composable, scriptable.

### Big architecture realization

You now basically have three interaction modes:
| Mode | Example |
|---|---|
| Deterministic CLI | `docker ps` |
| Command palette | fuzzy-search tools |
| Agentic orchestration | "fix unhealthy containers" |

### My updated architecture recommendation

```
Rust Core Runtime
├─ MCP subsystem
├─ ACP subsystem
├─ Schema engine
├─ Permission engine
├─ Search/index engine
└─ Session engine
Interfaces
├─ GUI (Tauri)
├─ CLI
├─ Agent runtime
└─ Optional web dashboard
```

---

## Turn 9

**You:** That also means that ANYONE can generate an extension based on an MCP server - or rather, we can just auto-generate them from the schema

**ChatGPT:** Exactly. That is the breakthrough insight. You do not want a traditional extension SDK. You want: MCP schema → automatically generated extension UX.

### This is the key inversion

Raycast: Developer writes extension UI manually.
Your app: Server exposes capabilities → Platform generates UX automatically. That is massively more scalable.

### Auto-generated from MCP metadata:

| Feature | Auto-generated? |
|---|---|
| Command palette entries | Yes |
| Forms | Yes |
| Validation | Yes |
| CLI flags | Yes |
| Detail pages | Yes |
| Search indexing | Yes |
| Permissions | Yes |
| Tool docs | Yes |
| Categories | Mostly |
| Quick actions | Yes |
| AI tool definitions | Yes |

### THIS is the actual moat

**Universal schema-driven UX generation** from MCP tools, ACP capabilities, JSON schema, metadata.

### Important implication

You should avoid custom extension frontend APIs as long as possible. Once you allow arbitrary frontend code/React extensions/arbitrary rendering, you lose consistency, security, determinism, maintainability.

### My strongest recommendation now

1. Schema engine (core IP)
2. Metadata normalization (also core IP)
3. Search/indexing (critical)
4. Permission/risk classification (very important)
5. Generated UX quality (this determines whether the platform feels magical or cheap)

---

## Turn 10

**You:** No marketplace becomes dead simple... Search the web for docs for claude code + codex marketplaces. (with attached Claude Code plugin/marketplace documentation)

**ChatGPT:** Yes. I think you're right. You probably should NOT build a traditional marketplace backend first. You should build: **A curated registry aggregator**.

Where "marketplaces" are literally just: git repos, manifests, indexes, MCP registries, SKILL.md repos, Claude/Codex marketplace repos.

### Claude Code already does this

Anthropic's official docs describe marketplaces as "a catalog" supporting git repositories and local paths.

```
/plugin marketplace add anthropics/skills
/plugin marketplace add ananddtyagi/cc-marketplace
```

### Your architecture simplifies dramatically

Instead of a centralized extension marketplace backend, you can do: registry/indexing/search layer over git-native ecosystems.

### Architecture layers

**Runtime:** MCP host, ACP host, schema renderer, command palette, session manager

**Registry layer:** Indexes Claude marketplaces, Codex skills repos, MCP registries, SKILL.md repos, GitHub repos, local manifests

**Search layer:** Normalizes metadata, schemas, tags, permissions, descriptions, aliases, categories

**Installer layer:** Installs MCP servers, skills, prompts, plugins, agents from git, local paths, manifests, package registries

### The ecosystem direction is becoming obvious

Git-native AI capability distribution. Not central app stores, but declarative manifests + git repos + registries.

### This also massively reduces your operational burden

You do NOT need for v1: payment infra, extension hosting, bundle hosting, extension sandbox runtime, custom SDK ecosystem, frontend extension API.

---

## Turn 11

**You:** Actually the more important thing would be to initially support official mcp registry - and we can just add more sources as needed

**ChatGPT:** Yes. That is the correct move. Official MCP registry as source of truth, then add your metadata overlay.

### Your metadata overlay

| MCP Registry Field | Your Added Metadata |
|---|---|
| name | aliases |
| description | curated summary |
| tools | generated categories |
| transport | security rating |
| auth | setup difficulty |
| tags | homelab/devops/ai/etc |
| version | compatibility notes |
| vendor fields | trust score |

### Another important thing — Registry adapters

```
RegistrySource trait/interface
Implementations:
- official MCP registry
- Anthropic marketplace
- Codex marketplace
- GitHub repo
- local manifest directory
- enterprise registry
```

### Internal model

```rust
Capability
Tool
Agent
Prompt
Resource
Server
RegistrySource
Installable
```

### One thing I would prioritize VERY early — Registry caching + indexing

Build: local SQLite index, Tantivy/meilisearch-style search, aggressive metadata normalization, background sync/update system.

Command-palette UX lives or dies on **instant discovery**, not just protocol support.

---

## Turn 12

**You:** AgentCast

**ChatGPT:** AgentCast is a much better name than "Labby" for this specific product direction.

| Quality | Assessment |
|---|---|
| Memorable | Strong |
| Broad enough | Yes |
| AI-native feel | Yes |
| Infrastructure feel | Yes |
| Protocol/runtime vibe | Yes |
| Sounds like a platform | Yes |
| Works for CLI/Desktop/Web | Yes |

### The framing I'd use

**AgentCast** — The desktop runtime for MCP tools and ACP agents.
or: A protocol-native command center for agents, tools, and automation.

### Naming scales naturally

- Desktop app: `agentcast`
- CLI: `agentcast`
- Runtime daemon: `agentcastd`
- Registry/index service: `AgentCast Registry`
- Enterprise/self-hosted: `AgentCast Hub`
- SDK/crates: `agentcast-mcp`, `agentcast-acp`, `agentcast-runtime`

### Architecture

```
AgentCast Runtime
├─ MCP Host
├─ ACP Host
├─ Registry Engine
├─ Schema Renderer
├─ Command Palette
├─ CLI Runtime
└─ Agent Session Manager
```

Do NOT market it initially as "AI launcher" — too weak, too crowded, too generic. Instead position around protocols and runtime: "Protocol-native desktop runtime", "Universal MCP/ACP runtime", "Command center for AI agents and tools".

---

## Turn 13

**You:** Well tbh I think we should develop cli/api/mcp first before we even begin to layer on the ui. 1) we can validate the model...

**ChatGPT:** I think that is the correct approach. The more this evolved, the less this looks like a desktop app project and the more it looks like a protocol/runtime project that happens to have a desktop shell.

### Your proposed order is the right order

```
core runtime → CLI → API → MCP server → desktop UI
```

### Architecture

```
agentcast/
├─ crates/
│  ├─ core/
│  ├─ runtime/
│  ├─ registry/
│  ├─ schema/
│  ├─ search/
│  ├─ policy/
│  ├─ acp/
│  ├─ mcp/
│  ├─ sessions/
│  └─ integrations/
├─ apps/
│  ├─ cli/
│  ├─ api/
│  ├─ mcp-server/
│  ├─ desktop/
│  └─ raycast-extension/
```

### Your point #3 is probably the most important engineering decision

"cli/api/mcp all just thin shims" — Exactly. The correct architecture.

**Core runtime should know NOTHING about:** Tauri, React, Raycast, terminals, HTTP, desktop UI. Instead: all interfaces consume the same runtime.

### Interface parity example

- CLI: `agentcast call docker`
- HTTP API: `POST /tools/docker/ps`
- MCP server: `{"tool": "docker.ps"}`
- Desktop UI: `Docker → List Containers`

ALL invoke the same internal capability execution path.

### Your point #4 is actually REALLY smart

The Raycast extension becomes a **distribution funnel**. Not the primary product.

### You are basically building: a capability hypervisor for MCP/ACP ecosystems.

---

## Turn 14

**You:** Well - labby does actually have a lot of that shit already implemented - BUT we didn't plan for it...

**ChatGPT:** AgentCast should not start from scratch. Labby is already 60–75% of the backend/runtime architecture you need.

### What Labby already gives AgentCast

**Directly reusable:**
- Shared core/service layer (lab-apis = pure SDK layer, lab = product runtime/adapters, lab-auth = OAuth/JWT/auth middleware)
- CLI/API/MCP dispatch contract (one shared execution path, CLI/MCP/API are sibling adapters)
- MCP gateway (stdio + remote HTTP upstreams, upstream auth/OAuth, exposure filters, protected MCP routes, tool search mode)
- Marketplace/registry foundation (Claude Code marketplace, Codex artifacts, MCP Registry, ACP Registry, local metadata overlay)
- Fleet/hub-spoke (master/non-master runtime, node enrollment, WebSocket reporting, remote deployment)
- ACP web/chat runtime
- Single binary / single port model
- Aurora design system (shadcn-compatible registry, command palette, permission prompts, AI blocks, file tree/editor, terminal, tool calls, auth/OAuth components)

### The key conclusion

AgentCast should probably be a generalized extraction/fork of Labby's runtime architecture, not a greenfield build.

- **Labby** = homelab control plane + MCP/ACP gateway + marketplace + fleet
- **AgentCast** = general MCP/ACP runtime + registry/stash + CLI/API/MCP + optional desktop UI

### What I would repurpose first

1. Dispatch layer model — keep "surface-neutral dispatch" rule
2. Gateway — strip homelab assumptions, keep upstream MCP proxying, OAuth upstreams, protected routes, tool filtering, tool search mode
3. Marketplace + stash — generalize to MCP servers, ACP agents, Claude plugins, Codex artifacts, skills, commands, hooks
4. Fleet/node runtime
5. Auth — keep OAuth/JWT/bearer split
6. Aurora — use as the desktop/web UI foundation

### Best next move

Create a serious extraction plan:
```
agentcast-core
agentcast-gateway
agentcast-registry
agentcast-stash
agentcast-fleet
agentcast-auth
agentcast-cli
agentcast-api
agentcast-mcp
agentcast-ui
```

My vote: new repo, but lift proven Labby patterns aggressively.

---

## Turn 15

**You:** Yeah def was gonna go with new repo - so currently labby is only 3 crates but p sure i have pretty good coverage already

**ChatGPT:** Yes. Extract crates first.

### Extraction order

1. **agent-runtime-core** — Shared types, envelopes, errors, operation metadata, IDs, config primitives.
2. **agent-dispatch** — Surface-neutral execution layer. This is the most important extraction.
3. **agent-mcp-gateway** — Upstream MCP proxying, OAuth upstreams, tool filtering, tool search mode, passthrough primitives.
4. **agent-registry** — Official MCP registry, ACP registry, Claude/Codex marketplace adapters, metadata overlays.
5. **agent-stash** — Artifact storage/sync model.
6. **agent-fleet** — Hub/spoke, node enrollment, websocket sessions, deploy/status/logs.
7. **agent-auth** — OAuth/JWT/bearer middleware, scopes, route protection.
8. **agent-ui-contracts** — Shared API DTOs for web/desktop/Raycast clients.

### Rule for extraction

A crate is worth extracting only if it can answer "yes" to this: "Could AgentCast use this without importing Plex/Arrs/Unraid/Gotify assumptions?"

### Strong opinion

The first crate should be `agentcast-core`. Then `agentcast-dispatch` and `agentcast-mcp-gateway`. Those three define the spine. Everything else can follow.

---

## Turn 16

**You:** Anything i build in rust I'm going to want to follow this pattern pretty much - well the architecture part anyway. So we wouldn't obviously need the gateway, stash, registry or fleet for everything

**ChatGPT:** Yes. That's the right extraction goal. You want two layers:

### 1. Reusable app framework crates

```
acore          # errors, envelopes, config, IDs, traits
adispatch      # surface-neutral operation dispatch
asurfaces-cli  # CLI adapter helpers
asurfaces-api  # Axum route helpers
asurfaces-mcp  # MCP exposure helpers
aweb-static    # static export serving + fallback routing
aauth          # OAuth/JWT/bearer middleware
adb            # SQLite migrations/pool helpers
aobservability # tracing/logging/redaction/correlation
ascaffold      # project/service generator
```

This is the "Jake Rust app pattern" toolkit.

### 2. Optional product/runtime crates

```
mcp-gateway
registry
stash
fleet
acp-web
marketplace
homelab-services
```

### The design rule

Every extracted feature crate should have:
- library crate first
- binary wrapper second

### The trait shape you want

```rust
trait Module {
    fn name(&self) -> &'static str;
    fn routes(&self) -> Router<AppState>;
    fn cli(&self) -> Command;
    fn mcp_tools(&self) -> Vec<ToolSpec>;
    fn migrations(&self) -> Vec<Migration>;
}

// Then any app can compose modules:
App::new()
    .with_auth(oauth)
    .with_sqlite()
    .with_module(GatewayModule)
    .with_module(RegistryModule)
    .with_static_web(admin_ui)
    .serve();
```

### Strong recommendation

Use neutral names for the framework crates. Then product crates can be branded:
- framework = reusable pattern
- AgentCast = product built on pattern
- Labby = product built on pattern

---

## Turn 17

**You:** Okay you have the GitHub app connected can you not use that to view my two repos?

**ChatGPT:** Yes — and that's what I ended up using. I was able to inspect your docs, repo structure, architecture docs, contracts, dispatch/runtime docs, gateway docs, marketplace/runtime docs through the GitHub-connected tooling.

**The important realization after reading your repo docs:** You already built the hard part.

Most people talking about MCP runtimes are still stuck at "toy MCP client." You already have:
- unified runtime
- dispatch abstraction
- OAuth-aware MCP gateway
- upstream MCP proxying
- ACP implementation
- marketplace aggregation
- node/fleet runtime
- stash/deployment model
- CLI/API/MCP parity
- local-first deployment
- static web bundling
- configurable runtime composition
- metadata overlays
- permissioning
- websocket fleet coordination

That's not a prototype anymore. That's a platform/runtime architecture.

The thing you're missing is: **proper extraction/generalization boundaries**, not features.

### The architecture I now think you should aim for

**Foundation framework crates** (reusable across ALL future projects):
```
runtime-core, runtime-auth, runtime-dispatch, runtime-api, runtime-mcp,
runtime-web, runtime-db, runtime-observability, runtime-config, runtime-scaffold
```

**Then optional runtime modules:** `mcp-gateway`, `registry`, `stash`, `fleet`, `marketplace`, `acp-runtime`

**Then product modules:** `labby-services`, `agentcast-app`, `homelab-marketplace`

---

## Turn 18

**You:** Yes when I learned you could use rust to do something like this the only other language that i use is SSH config + aliases

**ChatGPT:** That makes sense. Your architecture evolution reads like someone repeatedly applying "what is the smallest composable primitive that solves this cleanly?" over and over. That's basically Unix philosophy filtered through Rust, protocol runtimes, AI tooling, homelab ergonomics.

### The SSH config insight is genuinely underrated

Piggybacking on `~/.ssh/config` gives you: aliases, usernames, ports, jump hosts, identities, agent forwarding, ProxyCommand, existing operator workflows — for free. An extremely Unix-native move.

### General rule

**Prefer leveraging existing operator primitives over inventing new abstractions**

| Existing Primitive | Reuse Instead of Replacing |
|---|---|
| ~/.ssh/config | custom node inventory |
| git repos | custom marketplaces |
| JSON Schema | custom UI DSL |
| MCP Registry | custom plugin registry |
| OAuth | custom auth |
| SQLite | distributed infra |
| static exports | runtime frontend servers |
| stdio | proprietary IPC |
| WebSockets | heavy RPC stacks |

### The Aurora angle is actually strategically important too

You now potentially have: reusable operational runtime AND reusable UI runtime. That combination is extremely valuable long-term.

---

## Turn 19

**You:** Another surprising thing to learn is that you can effortlessly tunnel stdio mcp servers over ssh

**ChatGPT:** Yes. That's one of the most underappreciated properties of stdio MCP. You effectively discovered: **stdio + SSH = remote MCP transport** — without needing HTTP, OAuth, reverse proxies, API gateways, websocket infrastructure, custom auth, TLS management.

SSH already gives: encryption, auth, multiplexing, port forwarding, agent auth, jump hosts, auditability, battle-tested tooling.

```bash
ssh server "my-mcp-server"
# → already a remote MCP transport
```

### The really important implication

This means: **transport != deployment topology**. A stdio MCP server can be local, remote, containerized, tunneled, multiplexed, proxied — without the server even knowing.

### AgentCast should support SSH-backed MCP transports natively

```toml
[mcp.servers.remote]
transport = "ssh-stdio"
host = "homelab"
command = "labby mcp serve"
```

### Transport adapters

| Adapter | Description |
|---|---|
| local-stdio | local process |
| ssh-stdio | tunneled process |
| docker-exec | container process |
| kubectl-exec | k8s process |
| websocket | persistent remote |
| streamable-http | official remote |
| sidecar | embedded runtime |

---

## Turn 20

**You:** And on top of that we can also of course, tunnel acp over ssh.

**ChatGPT:** Yes — and that completes the pattern. SSH is a universal remote process substrate.

```
local process protocol
+ SSH process execution
= remote process protocol
```

This applies to: MCP over SSH stdio, ACP over SSH stdio, potentially any JSON-RPC-over-stdio protocol.

### Why ACP over SSH is especially powerful

MCP over SSH gives remote tools. ACP over SSH gives **remote agents with workspace locality**. That matters more.

Example: Run Codex on my workstation repo from my laptop. No need to copy the repo locally, no remote web service, no port exposure.

### Transport model

```
Protocol
├─ MCP
├─ ACP
└─ future JSON-RPC protocols
Transport
├─ local-stdio
├─ ssh-stdio
├─ streamable-http
├─ websocket
├─ docker-exec
├─ kubectl-exec
└─ agentcast-node
```

### Killer workflow

```
AgentCast: connect to devbox via SSH
→ start Codex ACP session
→ expose approved MCP tools from AgentCast
→ stream session UI locally
→ save transcript/session log to stash
→ deploy updated agent config back to devbox
```

**Headline capability: Run local agents and MCP servers anywhere you can SSH.**

---

## Turn 21

**You:** I mean we could almost easily make all this shit composable too. - You pick your ACP Agent - Plugins, Skills, Hooks, Subagents...

**ChatGPT:** Yes. That's the next abstraction. You're describing an **agent run composer**.

### The primitive: AgentProfile

```yaml
name: homelab-maintainer
agent:
  provider: claude-code
  version: latest
device: tootie
cwd: /mnt/user/projects/labby
prompt: Check service health, summarize issues, and propose safe fixes.
plugins:
  - labby/unraid
  - labby/docker
  - labby/gotify
skills:
  - homelab-troubleshooting
  - docker-log-analysis
subagents:
  - log-reader
  - deploy-reviewer
hooks:
  pre_run:
    - check_git_status
    - load_env
  on_tool_call:
    - require_approval_for_destructive_ops
  post_run:
    - save_session_log
    - send_gotify_summary
schedule:
  cron: "0 8 * * *"
policy:
  auto_approve:
    - read_logs
    - list_containers
  require_confirm:
    - restart_container
    - write_file
    - deploy
```

### This becomes "agent recipes"

```
homelab-health-check
repo-maintainer
release-manager
docs-updater
security-reviewer
dependency-upgrader
log-triage
unraid-operator
```

### The core object is: RunSpec

A RunSpec answers: Who runs? Where? With what tools? With what context? Under what policy? Triggered by what? What happens after?

### Minimal schema

```yaml
apiVersion: agentcast.dev/v1
kind: RunSpec
metadata:
  name: nightly-homelab-check
spec:
  agent:
    type: acp
    id: claude-code
  target:
    device: tootie
    cwd: /mnt/user/projects/homelab
  artifacts:
    plugins:
      - labby/unraid
      - labby/docker
    skills:
      - homelab-operator
    hooks:
      - gotify-summary
    subagents:
      - log-inspector
  prompt:
    inline: Check my homelab health. Do not make destructive changes without approval.
  tools:
    mcp:
      - labby.unraid
      - labby.docker
      - labby.gotify
  policy:
    approval:
      destructive: required
      filesystem_write: required
      read_only: auto
  trigger:
    schedule:
      cron: "0 8 * * *"
  output:
    sessionLog: true
    notify:
      gotify: true
```

### Separation

- **Artifact** — Reusable unit (skill, plugin, hook, command, subagent, mcp-server)
- **Profile** — Reusable environment (Claude Code + preferred plugins + default hooks + policies)
- **RunSpec** — Concrete execution (run this profile on this device in this cwd with this prompt now/scheduled)
- **Deployment** — Where artifacts/profiles are installed

**The product framing:** Run any agent, with any toolchain, on any device, from one reproducible spec.

---

## Turn 22

**You:** Damn - this escalated quickly lol, as do all my projects 😂

**ChatGPT:** Yeah. But this one escalated in a coherent direction. The through-line is strong:

- **Labby taught you:** CLI + API + MCP + web + OAuth + SQLite + single binary + one port
- **AgentCast adds:** ACP + agent profiles + stash + registry aggregation + remote execution + composable runs

The funny part is that the "Raycast for Windows" idea was almost immediately too small. The real thing is:

AgentCast = portable agent runtime + MCP/ACP gateway + artifact stash + run composer

**The danger:** scope gravity. You need a hard center of mass:

> AgentCast v0: Define, install, and run reproducible agent environments across devices.

### The sane next step

Start with: RunSpec + runtime + CLI

```bash
agentcast run homelab-health --device tootie --cwd /mnt/user/projects/labby
```

If that works, everything else becomes obvious: API wraps it, MCP exposes it, web visualizes it, Raycast triggers it, scheduler runs it, stash deploys it, Aurora makes it pretty.

---

## Turn 23

**You:** Oh so cli should host all that business?

**ChatGPT:** No. The CLI should NOT host the business logic. The CLI should be a thin shim over the shared runtime.

### Correct layering

```
agentcast-core / runtime
├─ business logic
├─ RunSpec execution
├─ registry/stash/fleet/gateway operations
├─ policy/approval engine
├─ transport adapters
└─ persistence

agentcast-cli
├─ parse args
├─ load config
├─ call runtime
└─ render output
```

### Rule

If logic would be useful from API, MCP, web, or scheduler, it **does not belong in CLI**.

- CLI owns: argument parsing, terminal UX, confirmations, color/output modes, exit codes
- Runtime owns everything else.

---

## Turn 24

**You:** This also gives you a pretty much fully portable operating plane for your agents and allows you to specify the minimum context for maximum efficiency

**ChatGPT:** Yes. That's exactly the value. **AgentCast is a portable operating plane for agent environments.**

### The token-saving angle is real

Most agent setups load everything: all plugins, all skills, all tools, all commands, all rules, all context — even when the task only needs a narrow subset.

AgentCast can instead resolve a task-specific environment that only includes: relevant skills, relevant tools, relevant prompts, relevant hooks, relevant working directory, relevant device context.

### Web adapter vs desktop app

```
Runtime
├─ CLI adapter
├─ API adapter
├─ MCP adapter
├─ Web adapter
├─ TUI adapter
└─ Desktop adapter
```

The desktop app should NOT be special. It is just: web UI + native shell capabilities.

**Best architecture:**
```
agentcast-runtime  → business logic
agentcast-api      → HTTP routes over runtime
agentcast-web      → static React/Aurora app over API
agentcast-desktop  → Tauri shell using agentcast-web UI
agentcast-cli      → terminal shim over runtime
agentcast-mcp      → MCP shim over runtime
```

---

## Turn 25

**You:** Well and just to put the cherry on top - couldn't we easily then take our composable and run it in a container?

**ChatGPT:** Yes. That drops out naturally from the architecture.

### Target backends

```yaml
target:
  kind: local | ssh | agentcast-node | docker | kubernetes
```

### Container targets give

| Benefit | Why it matters |
|---|---|
| Isolation | Agent cannot casually touch host files |
| Reproducibility | Same runner image everywhere |
| Dependency control | Pre-baked Rust/Node/Python/toolchains |
| Disposable runs | Throw away dirty environments |
| Safer experimentation | Good for untrusted plugins/tools |
| CI compatibility | Same RunSpec can execute in GitHub Actions later |

### The clean abstraction

```
ExecutionTarget (trait)
├─ LocalTarget
├─ SshTarget
├─ NodeTarget
├─ DockerTarget
├─ KubernetesTarget
└─ FirecrackerTarget
```

### Strong caution

**Design for it. Do not build it yet.** Containerized agent execution opens second-order complexity: image management, workspace mounting, secret injection, network policy, Docker socket exposure risks, cleanup, nested Docker, GPU/local device access.

---

## Turn 26

**You:** Yeah and then obviously we create a skill where you tell the agent what you want to do and the skill creates the loadout for you

**ChatGPT:** Yes. That is the obvious "meta-skill," and it should probably become one of AgentCast's flagship workflows.

### The primitive: Loadout Builder

User says: "I want a setup for maintaining my Rust MCP gateway project."

The skill inspects the user's stash: plugins, skills, MCP servers, hooks, subagents, commands, agents, devices, known projects, policies.

Then it emits one or more runnable loadouts: Rust Gateway Maintainer, Release Manager, Docs/Changelog Writer, Security Reviewer, Homelab Operator.

### Better: generate multiple loadout types

| Loadout | Purpose |
|---|---|
| dev-default | daily coding |
| debug-test-failure | focused test/log analysis |
| release-manager | changelog, tags, builds |
| security-review | auth, secrets, permission review |
| docs-maintainer | docs, examples, README updates |
| architecture-review | high-level crate/module analysis |

### User-facing naming

Internally: `Artifact`, `AgentProfile`, `RunSpec`, `Deployment`
User-facing: **Loadout**, **Run**, **Device**, **Stash**

"Loadout" is intuitive because it implies: *pick the exact gear for this mission.*

---

## Turn 27

**You:** Yup - and then agenthub - the github for your loadouts. A loadout repo would include one or multiple loadouts, which are like an agentic docker compose

**ChatGPT:** Yes. That is the cleanest packaging story by far. AgentHub should piggyback on Claude Code plugin repos, then add one orchestration file.

### The key move

A loadout repo should be: Claude Code plugin repo + `agent-compose.json` + `agent.loadout.yaml`. That's it.

### Proposed repo shape

```
my-agent-loadouts/
├── .claude-plugin/
│   ├── plugin.json
│   └── marketplace.json
├── agentcast.loadouts.yaml
├── skills/
│   ├── rust-mcp-maintainer/
│   │   └── SKILL.md
│   └── homelab-triage/
│       └── SKILL.md
├── agents/
│   ├── security-reviewer.md
│   └── release-manager.md
├── hooks/
│   ├── hooks.json
│   └── require-approval.sh
├── commands/
│   ├── release.md
│   └── health-check.md
├── monitors/
│   └── monitors.json
├── .mcp.json
├── .lsp.json
└── README.md
```

### "Agentic Docker Compose" is exactly the right analogy

`docker-compose.yml` does not define Docker itself. It orchestrates existing containers, networks, volumes, and env.

`agent.loadouts.yaml` should not define plugin packaging. It should orchestrate existing agent artifacts.

### AgentHub model

GitHub for loadout repos. Under the hood it can be dead simple: Git repo + plugin manifest + loadout yaml + metadata index.

### Strong recommendation

Do NOT invent an AgentCast plugin format. Invent only: `agent.loadouts.yaml`

**AgentCast is the composer, not the packaging authority.**

---

## Turn 28

**You:** Yeah and then agenthub could be more so about viewing, composing, editing loadouts and artifacts (with extensive Claude Code plugin/marketplace documentation pasted)

**ChatGPT:** Yeah. This is the moment where the product becomes much smaller and sharper. You do NOT need to invent a full artifact package system.

### Revised model

**1. `agent-loadout.json`** — Schema-compatible with Claude's `marketplace.json`. Lists available plugins/artifacts.

**2. `agent-compose.yaml`** — The orchestration layer. Answers: Which artifacts should be loaded, where should they run, under what policy, with what prompt, on what schedule?

### Minimal viable schema

```yaml
version: 1
runs:
  labby-review:
    agent: claude-code
    host: tootie
    cwd: /mnt/user/projects/lab
    plugins:
      - labby-rust-maintainer
      - mcp-gateway-tools
    prompt: Review the crate extraction plan.
```

**A YAML-driven remote agent runner that composes Claude/Codex plugin artifacts and tunnels ACP/MCP over SSH. That is a sharp v0.**

---

## Turn 29

**You:** Uh we would be supporting claude code marketplaces - we would use the exact same schema... All artifacts are claude plugins essentially

**ChatGPT:** Yes. That's the cleaner model.

### Correct split

**`agent-loadout.json`** — Claude Code marketplace schema-compatible. What plugins/artifacts are available, and where do they come from?

**`plugin.json`** — Standard Claude Code plugin manifest. What does this plugin contain?

**`agent-compose.yaml`** — The only real AgentCast-native layer. How do we assemble and run these artifacts?

### Your packaging rule is right

When users want to bundle individual skills/files, AgentCast creates a plugin for them. The artifact pipeline:

```
User selects skills/hooks/agents/MCP configs
→ AgentCast creates/updates plugin directory
→ Writes standard .claude-plugin/plugin.json
→ Adds/updates plugin entry in agent-loadout.json
→ References plugin in agent-compose.yaml run/loadout
→ Can run through AgentCast
→ Can still be installed by Claude Code
```

### The clean product statement

> AgentCast uses Claude Code marketplace/plugin format for artifact packaging, then adds `agent-compose.yaml` to define reproducible runs.

**AgentCast only owns orchestration.**

---

## Turn 30

**You:** well also - we can inline a lot of stuff too in the agent-loadout.json by setting the strictness level of plugins

**ChatGPT:** Yes. `strict: false` is the key that makes curated/inlined loadouts practical.

### Two clean modes

**1. Plugin-owned mode (`strict: true`)** — `plugin.json` is authoritative, marketplace entry supplements it.

**2. Curated loadout mode (`strict: false`)** — AgentCast / AgentHub owns how raw files are exposed. The marketplace entry IS the definition. `plugin.json` is not required, or must not conflict. Extremely useful for generated loadouts.

### Three artifact strategies

| Strategy | Best for |
|---|---|
| plugin-owned (`strict: true`) | normal third-party plugins |
| marketplace-owned (`strict: false`) | curated/generated loadouts |
| generated (`plugin.json`) | user-created plugin bundles |

### Clean division

```
agent-loadout.json  = artifact catalog + optional curated component exposure
agent-compose.yaml  = runtime orchestration
```

---

## Turn 31

**You:** Can you create detailed spec for everything we've defined / decided upon thus far

**ChatGPT:** *(Worked for 15 minutes)*

### AgentCast: Detailed Specification

**1. Overview and Purpose**

AgentCast is a local‑first runtime and orchestration environment for AI agents and tools. It harmonises Model Context Protocol (MCP) and Agent Client Protocol (ACP). By combining these protocols, AgentCast provides a single plane where tasks can be executed deterministically (direct tool calls), orchestrated (agent-driven), or both.

**2. Packaging and Distribution — Claude Code Compatibility**

AgentCast does NOT invent its own plugin format. It relies on the Claude Code plugin ecosystem for packaging.

`agent-loadout.json` — schema-compatible with `marketplace.json`. Defines the catalog of plugins/artifacts needed. Can set `strict: false` so marketplace definition serves as the full manifest.

**3. Orchestration with `agent-compose.yaml`**

Defines **runs** — reproducible tasks specifying which agent to launch, which artifacts to load, where to run, what policy, what prompt.

| Key | Description |
|---|---|
| agent | ACP agent type and identifier (e.g., claude-code, codex) |
| target | transport (local-stdio, ssh-stdio, docker, kubernetes), host, cwd |
| loadout | plugins, skills, agents, hooks, monitors, MCP servers |
| prompt | task/instruction string |
| policy | approvals for destructive/sensitive operations |
| schedule | optional cron/event-based trigger |
| hooks | lifecycle hooks (pre_run, post_run, on_tool_call) |
| env | environment variables |

**4. Runtime Architecture — Shared Core**

```
runtime-core, runtime-dispatch, runtime-api, runtime-mcp, runtime-acp,
runtime-auth, runtime-db, runtime-observability, runtime-web
```

**Optional Modules:**

| Module | Purpose |
|---|---|
| mcp-gateway | Proxies upstream MCP servers through runtime |
| registry | Aggregates official MCP registry and ACP registry |
| stash | Stores user-curated artifacts |
| fleet | Hub-spoke system for multi-device deployments |
| acp-runtime | Hosts ACP agents locally |

**Surface Adapters:**

| Adapter | Description |
|---|---|
| CLI (agentcast-cli) | Parses commands, delegates to dispatcher |
| API (agentcast-api) | HTTP JSON API |
| MCP Server (agentcast-mcp) | Exposes unified MCP endpoints |
| Web UI (agentcast-web) | React/TypeScript SPA with Aurora design system |
| Desktop (agentcast-desktop) | Tauri shell for hotkeys, notifications, offline mode |
| Raycast extension | Thin plugin connecting to local runtime |

**5. Transports and Execution**

- `local-stdio` — local process via stdin/stdout
- `ssh-stdio` — tunneled over SSH, leverages `~/.ssh/config`
- `docker` / `kubernetes` — (future) containerized runs

**6. Composability and Loadouts**

- **Loadouts** = reproducible combinations of artifacts plus runtime settings
- **Meta-skills** = special "loadout builder" skill auto-assembles candidate loadouts from stash
- **Policies** = approval gates for destructive/sensitive operations
- **Profiles vs runs** = profile is reusable environment, run binds profile to target
- **Recipes** = collections of runs for broader workflows

**7. User Interfaces**

- CLI, Web UI / Desktop (Aurora design system + Tauri + React), TUI, Raycast extension bridge

**8. Future Directions**

- Containerized runs, Advanced policy DSL, AgentCast Hub, AgentHub registry, Loadout versioning, Custom triggers and monitors

---

## Turn 32

**You:** OK - I just loaded you up with some context - use it to refine everything

**ChatGPT:** *(Worked for 8 minutes)*

### AgentCast Specification — Refined

**1. Purpose and High-level Goals**

AgentCast is a local-first runtime and orchestration environment for AI agents and tools. It harmonises MCP (tool/resource exposure) and ACP (coding agent sessions). By combining these, AgentCast provides a single plane where tasks can be executed deterministically, orchestrated (agent-driven), or both.

**2. Packaging — Claude Code Compatibility**

AgentCast does NOT invent its own plugin format. Uses:
- `.claude-plugin/plugin.json` (skills, agents, hooks, commands, mcpServers, lspServers, marketplaces)
- `marketplace.json` (catalog file listing available plugins)
- `agent-loadout.json` — schema-compatible with `marketplace.json`; entries can set `strict: false` to inline component definitions
- AgentCast can generate plugins on the fly when users bundle individual skills, hooks, or agents

**3. Orchestration with `agent-compose.yaml`**

Defines runs that specify agent, target, loadout, prompt, policy, schedule, hooks, env.

```yaml
version: 1
runs:
  homelab-health:
    agent:
      type: acp
      id: claude-code
    target:
      transport: ssh-stdio
      host: tootie
      cwd: /mnt/user/projects/homelab
    loadout:
      marketplace: jake-loadouts
      plugins: [ homelab-operator ]
      skills: [ homelab-troubleshooting ]
      agents: [ security-reviewer ]
      hooks: [ save-session-log, gotify-summary ]
    prompt: Check my homelab for unhealthy containers, summarize issues and propose safe fixes.
    policy:
      readOnly: auto
      destructive: require
      filesystemWrite: require
```

**4. Runtime and Crate Architecture**

Shared Core: runtime-core, runtime-dispatch, runtime-api, runtime-mcp, runtime-acp (in progress), runtime-auth, runtime-db, runtime-observability, runtime-web

Optional Modules: mcp-gateway, registry, stash, fleet, acp-runtime

Surface Adapters: agentcast-cli, agentcast-api, agentcast-mcp, agentcast-web (Aurora), agentcast-desktop (Tauri), agentcast-tui, Raycast extension

**5. Protocol Integrations**

**MCP:** Tool discovery via official registry, transports (stdio/Streamable HTTP), external auth (ext-auth), tool search mode (`tool_search` + `tool_execute`)

**ACP:** Session lifecycle (startSession, sendMessage, endSession), tool calls (`toolCall`), streaming and progress, registry (Claude Code, Codex, Gemini), transport (local-stdio, ssh-stdio, future docker)

**6. Transport Adapters**

- `local-stdio` — JSON-RPC over stdin/stdout
- `ssh-stdio` — wraps stdio in SSH session, leverages `~/.ssh/config`
- `docker`/`kubernetes` — (future) containerized isolation
- `agentcast-node` — websocket-based transport for fleet module

**7. Registry, Stash and Hub**

- Registry aggregation (official MCP, ACP, Claude/Codex marketplaces) + metadata normalization
- Stash — user-controlled store, versioned, deployable
- AgentHub — (future) web service for browsing, composing, versioning loadouts

**8. Composable Loadouts and Recipes**

Loadout = reusable package of artifacts. Run = combines loadout with target/prompt/policies. Recipes = common agent workflows stored as `agent-compose.yaml`.

**9. Aurora Design System and UI**

React/Tauri component library providing: command palette, forms generated from JSON Schema, permission prompts, file tree/diff viewer/logs/terminal, AI-SDK elements (blocks for agent messages, tool calls), themeable styling.

**10. Future Work**

Container isolation, advanced policies, AgentHub, extensibility via hooks, semantic search, Ops Mode, integration with other LLM runtimes.

**11. Conclusion**

AgentCast unifies MCP (tooling) and ACP (agents), providing a portable, reproducible and composable runtime for operating agent environments across devices, from a single specification file (`agent-compose.yaml`), while remaining fully compatible with the Claude Code plugin and marketplace ecosystem.
