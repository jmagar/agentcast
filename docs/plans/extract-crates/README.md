---
title: "Lab Extraction Bootstrap Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/acp/docs/markdown/0011-agentclientprotocol-com-protocol-initialization.md"
  - "docs/references/acp/docs/markdown/0018-agentclientprotocol-com-libraries-rust.md"
  - "docs/references/acp/docs/markdown/0022-agentclientprotocol-com-protocol-overview.md"
  - "docs/references/acp/docs/markdown/0028-agentclientprotocol-com-protocol-content.md"
  - "docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md"
  - "docs/references/acp/docs/markdown/0032-agentclientprotocol-com-protocol-file-system.md"
  - "docs/references/acp/docs/markdown/0043-agentclientprotocol-com-get-started-registry.md"
  - "docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md"
  - "docs/references/acp/docs/markdown/0062-agentclientprotocol-com-protocol-prompt-turn.md"
  - "docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md"
  - "docs/references/claude-code/markdown/0093-code-claude-com-docs-en-plugins.md"
  - "docs/references/claude-code/markdown/0102-code-claude-com-docs-en-plugins-reference.md"
  - "docs/references/claude-code/markdown/0108-code-claude-com-docs-en-plugin-marketplaces.md"
  - "docs/references/fastmcp/docs/auth.mdx"
  - "docs/references/fastmcp/docs/client.mdx"
  - "docs/references/fastmcp/docs/generate-cli.mdx"
  - "docs/references/fastmcp/docs/inspecting.mdx"
  - "docs/references/fastmcp/docs/overview.mdx"
  - "docs/references/fastmcp/docs/running.mdx"
  - "docs/references/jmagar/jmagar-aurora-design-system.xml"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
  - "docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md"
  - "docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md"
  - "docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md"
  - "docs/references/mcp/docs/markdown/0185-modelcontextprotocol-io-docs-learn-architecture.md"
  - "docs/references/mcporter/docs/agent-skills.md"
  - "docs/references/mcporter/docs/cli-reference.md"
  - "docs/references/mcporter/docs/livetests.md"
  - "docs/references/mcporter/docs/mcp.md"
  - "docs/references/mcporter/docs/spec.md"
  - "docs/references/mcporter/docs/tool-calling.md"
related:
  - "docs/plans/extract-crates/agent-acp.md"
  - "docs/plans/extract-crates/agent-api.md"
  - "docs/plans/extract-crates/agent-auth.md"
  - "docs/plans/extract-crates/agent-cli.md"
  - "docs/plans/extract-crates/agent-config.md"
  - "docs/plans/extract-crates/agent-core.md"
  - "docs/plans/extract-crates/agent-fleet.md"
  - "docs/plans/extract-crates/agent-gateway.md"
  - "docs/plans/extract-crates/agent-marketplace.md"
  - "docs/plans/extract-crates/agent-mcp.md"
  - "docs/plans/extract-crates/agent-observability.md"
  - "docs/plans/extract-crates/agent-protocol.md"
  - "docs/plans/extract-crates/agent-registry.md"
  - "docs/plans/extract-crates/agent-runtime.md"
  - "docs/plans/extract-crates/agent-schema.md"
  - "docs/plans/extract-crates/agent-search.md"
  - "docs/plans/extract-crates/agent-server.md"
  - "docs/plans/extract-crates/agent-stash.md"
  - "docs/plans/extract-crates/agent-store.md"
  - "docs/plans/extract-crates/agent-ui-contracts.md"
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Lab Extraction Bootstrap Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bootstrap AgentCast by extracting and generalizing proven Lab runtime, gateway, registry, marketplace, API, CLI, and UI patterns into AgentCast crate boundaries.

**Architecture:** Treat Lab as source material, not as a dependency. Extract only the generic behavior that fits AgentCast's MVP and crate map, then prove each slice with AgentCast-owned tests and fixtures.

**Tech Stack:** Rust 2024 workspace, Lab local repo, Lab Repomix snapshots, MCP, ACP, RMCP, agent-client-protocol, Axum, Clap, rusqlite, Tokio.

---

## Source Of Truth

Use these sources when planning extraction:

- `../lab`: live local Lab repo when available.
- `docs/references/jmagar/jmagar-lab.xml`: Repomix source-of-truth snapshot for Lab at the captured revision.
- `docs/references/jmagar/jmagar-aurora-design-system.xml`: Repomix source-of-truth snapshot for Aurora design system at the captured revision.
- `docs/reports/lab-extraction-source-map.md`: first-party source map from the live `../lab` exploration pass.

When `../lab` and the repopack disagree, inspect the live repo and decide whether the repopack is stale. Do not assume folder names or seed transcript claims are accurate without checking source.

Do not modify `docs/references/` while executing these plans. Reference snapshots, repopacks, and seed transcripts preserve source/provenance text.

## Upstream Reference Checks

- Lab source claims must be checked against `docs/references/jmagar/jmagar-lab.xml`; Aurora UI claims must be checked against `docs/references/jmagar/jmagar-aurora-design-system.xml`.
- MCP launcher claims must be checked against `docs/references/mcp/docs/markdown/0185-modelcontextprotocol-io-docs-learn-architecture.md`, `docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md`, `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`, `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`, `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`, `docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md`, and `docs/references/mcp/docs/markdown/0111-modelcontextprotocol-io-specification-2025-11-25-schema.md`.
- MCP Registry and install-package claims must be checked against `docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md` and `docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md`; the registry is preview, so plans must not promise GA stability.
- ACP claims must be checked against `docs/references/acp/docs/markdown/0022-agentclientprotocol-com-protocol-overview.md`, `docs/references/acp/docs/markdown/0011-agentclientprotocol-com-protocol-initialization.md`, `docs/references/acp/docs/markdown/0062-agentclientprotocol-com-protocol-prompt-turn.md`, `docs/references/acp/docs/markdown/0028-agentclientprotocol-com-protocol-content.md`, `docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md`, `docs/references/acp/docs/markdown/0032-agentclientprotocol-com-protocol-file-system.md`, `docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md`, `docs/references/acp/docs/markdown/0018-agentclientprotocol-com-libraries-rust.md`, and `docs/references/acp/docs/markdown/0043-agentclientprotocol-com-get-started-registry.md`.
- Claude Code plugin and MCP compatibility claims must be checked against `docs/references/claude-code/markdown/0091-code-claude-com-docs-en-mcp.md`, `docs/references/claude-code/markdown/0093-code-claude-com-docs-en-plugins.md`, `docs/references/claude-code/markdown/0102-code-claude-com-docs-en-plugins-reference.md`, and `docs/references/claude-code/markdown/0108-code-claude-com-docs-en-plugin-marketplaces.md`.
- FastMCP and mcporter are comparison references, not AgentCast dependencies in these plans: use `docs/references/fastmcp/docs/overview.mdx`, `docs/references/fastmcp/docs/client.mdx`, `docs/references/fastmcp/docs/running.mdx`, `docs/references/fastmcp/docs/inspecting.mdx`, `docs/references/fastmcp/docs/generate-cli.mdx`, `docs/references/fastmcp/docs/auth.mdx`, `docs/references/mcporter/docs/spec.md`, `docs/references/mcporter/docs/tool-calling.md`, `docs/references/mcporter/docs/cli-reference.md`, `docs/references/mcporter/docs/mcp.md`, `docs/references/mcporter/docs/agent-skills.md`, and `docs/references/mcporter/docs/livetests.md` when implementing comparable client, CLI, auth, inspection, or live-test behavior.

## Extraction Rule

Extract code and patterns only when they can be generalized without carrying Lab-specific service assumptions.

Allowed to extract/generalize:

- surface-neutral runtime/dispatch patterns.
- MCP gateway upstream routing, filtering, health, and tool search patterns.
- registry and metadata normalization patterns.
- marketplace/install-plan patterns.
- auth primitives when not tied to Lab service policy.
- API/CLI thin-surface patterns.
- static web/admin composition patterns.
- Aurora UI components that fit AgentCast surfaces.
- persistence, logging, and observability helpers when generic.

Do not extract into AgentCast core:

- Plex/Sonarr/Radarr/Unraid/Gotify workflows.
- homelab service SDKs.
- service credential extraction from appdata.
- Lab-only node/fleet policy unless promoted into a post-v0 AgentCast spec.
- UI copy or flows that assume Lab service administration.

## MVP Extraction Priority

For the MCP launcher MVP, extract or recreate in this order:

1. config patterns for MCP upstreams.
2. MCP stdio/server lifecycle handling.
3. tool discovery and normalized catalog generation.
4. gateway catalog merge and collision behavior.
5. deterministic tool invocation path.
6. CLI/API thin-surface patterns.
7. registry search/cache/normalization for MCP servers.
8. install-plan preview/apply patterns.

ACP, loadouts, stash, fleet, desktop UI, and remote execution remain post-v0 unless explicitly promoted in `docs/MVP.md`.

## Crate Plans

- [agent-acp.md](./agent-acp.md)
- [agent-api.md](./agent-api.md)
- [agent-auth.md](./agent-auth.md)
- [agent-cli.md](./agent-cli.md)
- [agent-config.md](./agent-config.md)
- [agent-core.md](./agent-core.md)
- [agent-gateway.md](./agent-gateway.md)
- [agent-marketplace.md](./agent-marketplace.md)
- [agent-mcp.md](./agent-mcp.md)
- [agent-observability.md](./agent-observability.md)
- [agent-protocol.md](./agent-protocol.md)
- [agent-registry.md](./agent-registry.md)
- [agent-runtime.md](./agent-runtime.md)
- [agent-schema.md](./agent-schema.md)
- [agent-search.md](./agent-search.md)
- [agent-server.md](./agent-server.md)
- [agent-store.md](./agent-store.md)
- [agent-stash.md](./agent-stash.md)
- [agent-fleet.md](./agent-fleet.md)
- [agent-ui-contracts.md](./agent-ui-contracts.md)

## Verification Expectations

Every extracted slice should include:

- source path or repopack evidence from Lab.
- what was kept.
- what was renamed/generalized.
- what Lab-specific behavior was intentionally left behind.
- tests or fixtures proving the AgentCast behavior independently of Lab.

Extraction is not copying blindly. The output must match AgentCast crate boundaries and MVP scope.

## Test Placement Rule

Use source-side unit tests for these extraction plans. Implementation files may declare `#[cfg(test)] mod tests;` when needed, but test bodies belong in source-side sidecar files such as `src/foo/tests.rs`, `src/foo_test.rs`, or `src/foo/tests/*.rs`. Do not create `crates/*/tests/` integration-test files for crate-internal behavior; reserve integration tests only for explicit cross-crate public contracts.

## Work Plan

### Task 1: Confirm Target Scope Before Each Extraction

**Files:**
- Read: `docs/MVP.md`
- Read: `docs/CRATE_BOUNDARIES.md`
- Read: `docs/plans/extract-crates/README.md`
- Read: the relevant `docs/plans/extract-crates/agent-*.md`

- [ ] **Step 1: Read the MVP and crate boundary docs.**

Run:

```bash
sed -n '1,220p' docs/MVP.md
sed -n '1,260p' docs/CRATE_BOUNDARIES.md
```

Expected: the target slice is confirmed as MCP launcher v0 unless the crate plan explicitly marks post-v0 work.

- [ ] **Step 2: List the crate plans and read the one being executed.**

Run:

```bash
find docs/plans/extract-crates -maxdepth 1 -type f -name 'agent-*.md' -print | sort
sed -n '1,260p' docs/plans/extract-crates/agent-acp.md
```

Expected: the implementation boundary, Lab sources, and verification commands are known before editing code. The second command is the concrete ACP example; use the matching listed file for a different crate.

### Task 2: Collect Evidence From Lab

**Files:**
- Read: `../lab`
- Read: `docs/references/jmagar/jmagar-lab.xml`

- [ ] **Step 1: Locate candidate Lab files.**

Run:

```bash
rg --files ../lab | rg 'acp|gateway|marketplace|mcpregistry|config|auth|cli|api|runtime'
```

Expected: candidate source files are identified from the live repo first.

- [ ] **Step 2: Compare against the Lab repopack when live source is unclear.**

Run:

```bash
rg -n 'AcpSessionRegistry|GatewayManager|mcpregistry|AcpEvent|install plan' docs/references/jmagar/jmagar-lab.xml
```

Expected: stale or missing live source is resolved against the captured repopack.

### Task 3: Implement In AgentCast Boundaries

**Files:**
- Modify: `crates/agent-acp/src/lib.rs` for the concrete ACP example.
- Create: `crates/agent-acp/src/events.rs` for the concrete ACP example.
- Test sidecars: `crates/agent-acp/src/{content,events,permissions,provider,session}.rs` for the concrete ACP example.

- [ ] **Step 1: Write the AgentCast-owned tests first.**

Run:

```bash
cargo nextest run -p agent-acp
```

Expected: the new behavior fails before implementation or the existing placeholder crate confirms no matching tests exist yet.

- [ ] **Step 2: Extract the smallest generic implementation.**

Expected: implementation uses AgentCast names, error types, and crate dependencies; Lab-specific service names and homelab behavior do not cross the boundary.

- [ ] **Step 3: Re-run focused verification.**

Run:

```bash
cargo nextest run -p agent-acp
```

Expected: focused tests pass.

### Task 4: Record The Extraction Decision

**Files:**
- Modify: the relevant `docs/plans/extract-crates/agent-*.md`
- Create or modify when behavior becomes stable: `docs/contracts/*.md`
- Create or modify when implementation detail matters: `docs/specs/*.md`

- [ ] **Step 1: Update the crate plan with evidence.**

Expected: the plan lists Lab source paths, kept behavior, renamed/generalized behavior, left-behind behavior, and verification evidence.

- [ ] **Step 2: Promote stable behavior into contracts or specs.**

Expected: contracts contain testable invariants; specs contain implementation details and edge cases.
