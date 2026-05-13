---
title: "AgentCast Docs"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/jmagar/jmagar-lab.xml"
related:
  - "docs/ARCH.md"
  - "docs/CLI.md"
  - "docs/CODE_ORGANIZATION.md"
  - "docs/DEVELOPMENT.md"
  - "docs/DEV_SPEED.md"
  - "docs/CRATE_BOUNDARIES.md"
  - "docs/ERRORS.md"
  - "docs/INSTALL_PLANS.md"
  - "docs/LAUNCHER.md"
  - "docs/MCP_RUNTIME.md"
  - "docs/MVP.md"
  - "docs/OBSERVABILITY.md"
  - "docs/PRINCIPLES.md"
  - "docs/PROTOCOLS.md"
  - "docs/QUALITY_GATES.md"
  - "docs/ROADMAP.md"
  - "docs/RUNTIME.md"
  - "docs/SCHEMA_UX.md"
  - "docs/SECURITY.md"
  - "docs/TESTING.md"
  - "docs/contracts/README.md"
  - "docs/examples/README.md"
  - "docs/plans/extract-crates/README.md"
  - "docs/specs/README.md"
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# AgentCast Docs

AgentCast is an agent runtime, registry, marketplace, and gateway for ACP/MCP-compatible coding agents and tool systems.

The target architecture is broad, but the first implementation slice is intentionally narrow: an MCP-first launcher/runtime. [MVP.md](./MVP.md) is the sequencing contract for that first slice.

These docs are development contracts. They are not marketing docs. When code and docs disagree, either the code is wrong or the contract needs an explicit design update.

## Read First

1. [MVP.md](./MVP.md) — MCP launcher scope and build order for v0.
2. [PRINCIPLES.md](./PRINCIPLES.md) — decision filter extracted from the seed conversations.
3. [ROADMAP.md](./ROADMAP.md) — phase gates from MVP to end-state architecture.
4. [ARCH.md](./ARCH.md) — end-state product shape and core architecture.
5. [CRATE_BOUNDARIES.md](./CRATE_BOUNDARIES.md) — what each crate owns and must not own.
6. [CODE_ORGANIZATION.md](./CODE_ORGANIZATION.md) — file/module size rules and simplicity expectations.
7. [LAUNCHER.md](./LAUNCHER.md) — launcher action product contract.
8. [MCP_RUNTIME.md](./MCP_RUNTIME.md) — MCP server lifecycle/runtime behavior.
9. [CLI.md](./CLI.md) — CLI commands, output rules, and exit codes.
10. [INSTALL_PLANS.md](./INSTALL_PLANS.md) — registry-to-install-plan flow.
11. [SECURITY.md](./SECURITY.md) — local MCP execution trust model and gates.
12. [SCHEMA_UX.md](./SCHEMA_UX.md) — JSON Schema input and future form behavior.
13. [ERRORS.md](./ERRORS.md) — normalized error taxonomy.
14. [RUNTIME.md](./RUNTIME.md) — the central business/orchestration layer.
15. [PROTOCOLS.md](./PROTOCOLS.md) — ACP/MCP ownership and adapter rules.
16. [OBSERVABILITY.md](./OBSERVABILITY.md) — local logs/events/audit expectations.
17. [TESTING.md](./TESTING.md) — verification gates and expectations.
18. [DEVELOPMENT.md](./DEVELOPMENT.md) — local development setup and command loop.
19. [DEV_SPEED.md](./DEV_SPEED.md) — nextest, Cranelift, and fast local workflows.
20. [QUALITY_GATES.md](./QUALITY_GATES.md) — pre-commit, pre-push, merge, and release gates.
21. [plans/extract-crates/README.md](./plans/extract-crates/README.md) — how to use Lab as the bootstrap source without importing Lab-specific product gravity.

## Contracts And Specs

- [contracts/](./contracts/README.md) contains stable, testable behavior contracts.
- [specs/](./specs/README.md) contains implementation specs that satisfy those contracts.
- [examples/](./examples/README.md) contains concrete v0 shapes for configs, actions, invocations, and install plans.

## Product Boundary

AgentCast owns:

- the MCP launcher/runtime MVP described in [MVP.md](./MVP.md).
- ACP agent sessions as post-v0/end-state behavior.
- MCP server/client/gateway behavior.
- official MCP Registry discovery for v0.
- ACP Registry and Claude Code plugin marketplace discovery as post-v0/end-state behavior.
- local agent process supervision.
- workspace/session/event state.
- install plans for MCP servers in v0.
- install plans for Claude plugins, ACP agents, and other artifact ecosystems only when their upstream references or AgentCast decisions define the supported format.
- web/API/CLI surfaces for the above.

AgentCast does not own:

- homelab service SDKs.
- media-service operation catalogs.
- Plex/Sonarr/Radarr/Unraid-specific workflows.
- service credential extraction from appdata.

Those belong in Labby or future optional plugins.

## Bootstrap Source

AgentCast is a separate repo, but it should bootstrap from proven Lab code and patterns where they can be generalized cleanly. Use `../lab` and `docs/references/jmagar/jmagar-lab.xml` as source material for extraction, following [plans/extract-crates/README.md](./plans/extract-crates/README.md).

## Golden Rule

Surface crates parse input and render output. Runtime/domain crates make decisions.

```txt
CLI/API/UI -> runtime/gateway/marketplace/registry -> protocol adapters -> external systems
```

No product decision should live only in CLI, HTTP routes, or UI code.

## Roadmap Discipline

End-state contracts should stay in place. If a doc describes ACP, loadouts, marketplace artifacts, remote execution, or AgentHub behavior, treat it as the intended architecture unless [MVP.md](./MVP.md) says it is out of scope for v0.
