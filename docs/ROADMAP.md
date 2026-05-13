---
title: "Roadmap"
doc_type: "guide"
status: "active"
owner: "agentcast"
audience:
  - "contributors"
scope: "end-state"
source_of_truth: true
upstream_refs: []
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "fe10007"
review_basis: "cross-referenced against local docs/references snapshot"
---

# Roadmap

This roadmap sequences AgentCast from the MCP launcher MVP to the broader agent operating-plane shape. It is implementation-oriented; product ideas that are not in a phase gate do not block earlier phases.

## Phase 0: Docs, Contracts, And Crate Boundaries

Goal: make the first build path unambiguous.

Required artifacts:

- `docs/MVP.md`
- `docs/CRATE_BOUNDARIES.md`
- `docs/contracts/`
- `docs/specs/`
- `docs/plans/extract-crates/`

Exit criteria:

- launcher action, MCP runtime, CLI, install-plan, schema UX, security, and error contracts exist.
- crate extraction plans point to Lab evidence when Lab is the source.
- post-v0 ACP/loadout/remote/UI concepts are preserved but not required for v0.

## Phase 1: CLI MCP Launcher

Goal: run the complete launcher loop from the CLI with local stdio MCP servers.

Required capabilities:

- load AgentCast config.
- start enabled local stdio MCP servers.
- initialize MCP clients.
- discover tools.
- normalize tools into `LauncherAction` entries.
- list/search actions.
- invoke an action by stable action id.
- render human and JSON output.
- shut down child processes cleanly.

Exit criteria:

- `agentcast servers list` reports configured server health.
- `agentcast actions list` shows discovered actions.
- `agentcast call <action-id> --json-args ...` invokes the owning MCP tool.
- tests cover discovery, collision handling, invocation, shutdown, and normalized errors without network by default.

## Phase 2: Registry Install

Goal: install MCP servers through previewable plans.

The official MCP Registry is the v0 source. It is a preview upstream service, so this phase must use a local cache/index, preserve provenance/freshness/status, and pass fixture-based tests without requiring network access.

Required capabilities:

- query or load official MCP Registry entries.
- normalize registry entries into install candidates.
- generate an install-plan preview.
- detect config conflicts.
- apply the plan through runtime/config APIs.
- verify the installed server can initialize or report a useful failure.

Exit criteria:

- registry search works from fixture tests without network.
- install-plan preview never mutates disk.
- apply writes config through the same mutation path used by CLI/manual config flows.
- failed verification leaves a structured error and does not silently claim success.

## Phase 3: API And Gateway Surfaces

Goal: expose the proven runtime path over HTTP and MCP gateway surfaces.

Required capabilities:

- thin HTTP routes for servers, launcher actions, invocations, registry, install plans, and health.
- one MCP gateway surface that projects the same action catalog.
- shared error envelopes.
- auth policy for non-loopback HTTP exposure.

Exit criteria:

- API and MCP surfaces delegate to runtime/gateway code rather than duplicating discovery or invocation.
- API JSON output matches the documented contracts.
- destructive or high-risk operations require confirmation gates.

## Phase 4: Web UI

Goal: add a command-palette-like web surface over the same runtime path.

Required capabilities:

- action search/list UI.
- schema-driven invocation forms.
- server health and logs views.
- install-plan preview/apply UI.
- basic history and favorites if the runtime metadata is already present.

Exit criteria:

- UI uses API/runtime semantics and does not implement its own MCP client.
- schema fallback paths are visible for unsupported input shapes.
- generated forms match `docs/SCHEMA_UX.md`.

## Phase 5: ACP, Loadouts, And Agent Operating Plane

Goal: promote post-v0 architecture after the MCP launcher is stable.

Candidate capabilities:

- ACP provider/session support.
- ACP Registry and ACP provider install plans.
- loadouts.
- stash/history/workspace workflows.
- remote execution targets.
- AgentHub/marketplace expansion.

Exit criteria:

- ACP work follows `docs/plans/extract-crates/agent-acp.md`.
- ACP session ownership lives in runtime, not adapter crates.
- loadouts and remote execution have contracts before implementation.
