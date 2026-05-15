---
title: "agent-acp Extraction Implementation Plan"
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
  - "docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md"
  - "docs/references/acp/docs/markdown/0062-agentclientprotocol-com-protocol-prompt-turn.md"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0185-modelcontextprotocol-io-docs-learn-architecture.md"
related: []
last_reviewed: "2026-05-15"
last_modified: "2026-05-15"
modified_on_branch: "gateway-first-skeleton"
modified_at_version: "0.1.0"
modified_at_commit: "d327495"
review_basis: "cross-referenced against gateway-first implementation audit and local docs/references snapshot"
---

# agent-acp Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `agent-acp` as the ACP protocol adapter crate: ACP SDK integration, ACP-to-AgentCast conversions, provider-session command wrappers, permission conversion, and event normalization without owning AgentCast session registry or runtime policy.

**Architecture:** Lab proves the ACP surface, but its implementation mixes reusable ACP protocol work with Lab runtime, API, UI, persistence, and marketplace concerns. AgentCast should put protocol-neutral event/session models in `agent-protocol`, ACP SDK conversions and provider wrappers in `agent-acp`, and session ownership/process supervision/persistence in `agent-runtime`.

**Tech Stack:** Rust 2024, `agent-client-protocol`, Tokio channels, serde/serde_json, thiserror, jiff, uuid, AgentCast protocol models.

---

## MVP Position

`agent-acp` is post-v0 for the MCP launcher MVP. Create the boundary now so the end-state docs remain coherent, but do not let ACP sessions, provider switching, chat UI, ACP Registry, or ACP Marketplace targets block `docs/MVP.md`.

The immediate code extraction priority remains `agent-config`, `agent-mcp`, `agent-runtime`, `agent-gateway`, `agent-cli`, `agent-registry`, and `agent-marketplace` for MCP launcher behavior.

## Current Implementation Audit

As of 2026-05-15 on `gateway-first-skeleton`, `agent-acp` implements the first post-v0 adapter contract layer: ACP adapter errors, provider event normalization DTOs, permission option/decision DTOs, and provider-session command/request DTOs. ACP runtime launch, provider subprocess supervision, persistence, registry/marketplace behavior, and chat UI remain outside this crate until post-v0 work promotes them.

## Lab Evidence Read

Use these Lab files as extraction evidence:

- `../lab/docs/acp/README.md`
- `../lab/docs/acp/design.md`
- `../lab/docs/acp/research-findings.md`
- `../lab/docs/acp/chat-session-persistence-investigation-2026-05-05.md`
- `../lab/Cargo.toml`
- `../lab/crates/lab-apis/src/acp.rs`
- `../lab/crates/lab-apis/src/acp/types.rs`
- `../lab/crates/lab-apis/src/acp/error.rs`
- `../lab/crates/lab-apis/src/acp/session.rs`
- `../lab/crates/lab-apis/src/acp/persistence.rs`
- `../lab/crates/lab-apis/tests/acp_types.rs`
- `../lab/crates/lab/src/acp/types.rs`
- `../lab/crates/lab/src/acp/providers.rs`
- `../lab/crates/lab/src/acp/runtime.rs`
- `../lab/crates/lab/src/acp/registry.rs`
- `../lab/crates/lab/src/dispatch/acp/catalog.rs`
- `../lab/crates/lab/src/dispatch/acp/dispatch.rs`
- `../lab/crates/lab/src/dispatch/acp/params.rs`
- `../lab/crates/lab/src/dispatch/acp/page_context.rs`
- `../lab/crates/lab/src/dispatch/acp/persistence.rs`
- `../lab/crates/lab/tests/acp_backend_contract.rs`
- `../lab/crates/lab/tests/acp_docker_provider_config.rs`
- `../lab/crates/vendor/agent-client-protocol/Cargo.toml`
- `../lab/crates/vendor/agent-client-protocol/src/schema/mod.rs`
- `../lab/crates/vendor/agent-client-protocol/src/session.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`, including the vendored `crates/vendor/agent-client-protocol/**` files present in that snapshot.
- ACP protocol claims are cross-checked against `docs/references/acp/docs/markdown/0022-agentclientprotocol-com-protocol-overview.md`, `docs/references/acp/docs/markdown/0011-agentclientprotocol-com-protocol-initialization.md`, `docs/references/acp/docs/markdown/0062-agentclientprotocol-com-protocol-prompt-turn.md`, `docs/references/acp/docs/markdown/0028-agentclientprotocol-com-protocol-content.md`, `docs/references/acp/docs/markdown/0031-agentclientprotocol-com-protocol-tool-calls.md`, `docs/references/acp/docs/markdown/0032-agentclientprotocol-com-protocol-file-system.md`, `docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md`, and `docs/references/acp/docs/markdown/0018-agentclientprotocol-com-libraries-rust.md`.
- MCP hand-off claims are cross-checked against `docs/references/mcp/docs/markdown/0185-modelcontextprotocol-io-docs-learn-architecture.md` and `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`.

## Live Lab Findings

- Lab ACP work spans `crates/lab-apis/src/acp*`, `crates/lab/src/acp/*`, `crates/lab/src/dispatch/acp/*`, and `apps/gateway-admin/lib/acp/*`.
- Protocol-neutral session/event/content/permission DTOs belong in `agent-protocol`; ACP SDK conversions and provider command wrappers belong in `agent-acp`.
- Lab's session registry, persistence, process supervision, turn-drain loops, and provider environment policy are extraction evidence for `agent-runtime` and `agent-store`, not `agent-acp`.
- ACP Registry and ACP Marketplace behavior is post-v0 and should not block the MCP launcher MVP.

## Extraction Boundary

Extract into `agent-acp`:

- ACP SDK dependency setup and version/patch verification from `../lab/Cargo.toml`.
- conversions from `agent_client_protocol::schema::*` into AgentCast ACP protocol models.
- prompt attachment conversion into ACP `ContentBlock` values, including local text/blob resources and percent-encoded local attachment URIs.
- session command wrappers for prompt, cancel, permission approval, and permission rejection.
- permission option conversion from ACP SDK `PermissionOptionKind` into AgentCast semantic option kinds.
- normalized provider health and provider model extraction from ACP initialize/session responses.
- event normalization for message chunks, reasoning chunks, tool calls, tool call updates, permission requests/outcomes, usage updates, content blocks, session info, config updates, modes, plans, and unknown provider notifications.
- raw payload preservation rules for `usage_update`, `ContentBlock[]`, provider info, `_meta`, raw tool input, raw tool output, and unknown variants.
- display-terminal metadata preservation as raw provider info, without executing terminal sessions.
- tests and fixtures that prove conversion behavior without launching Codex, Claude, or any external ACP process.

Keep out of `agent-acp`:

- session registry ownership, session list/get/start/close semantics, event sequence stamping, event replay, and principal checks. Those belong in `agent-runtime`.
- OS subprocess launch, process groups, shutdown timeouts, turn-drain loops, provider EOF classification, and per-session thread/process ownership. Those belong in `agent-runtime`.
- SQLite persistence, HMAC signing, redaction at storage boundaries, and SSE backfill. Those belong in `agent-runtime` or a future storage-focused module.
- shared action catalog, `action + params` dispatch, CLI syntax, HTTP routes, and MCP tool projection. Those belong in `agent-gateway`, `agent-cli`, `agent-api`, and `agent-server`.
- ACP Registry install and provider install-plan behavior. Those belong in `agent-registry` and `agent-marketplace`.
- Lab browser chat render models and gateway-admin UI components.
- Lab-specific environment variables such as `LAB_ACP_DB`, `LAB_ACP_HMAC_SECRET`, `LAB_ACP_SESSION_DIR`, `LAB_ACP_PROMPT_IDLE_TIMEOUT_MS`, `LAB_ACP_PERMISSION_TIMEOUT_MS`, `ACP_CODEX_COMMAND`, and `ACP_CODEX_ARGS`.
- Lab's `Bridge*` compatibility projections except as migration evidence for what not to name in AgentCast.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-acp/src/lib.rs` - public module exports; remove the generated `add` example.
- Create: `crates/agent-acp/src/error.rs` - ACP adapter error type and conversions.
- Create: `crates/agent-acp/src/content.rs` - ACP content block and prompt attachment conversions.
- Create: `crates/agent-acp/src/events.rs` - ACP provider update/event normalization.
- Create: `crates/agent-acp/src/permissions.rs` - permission option and decision conversions.
- Create: `crates/agent-acp/src/provider.rs` - provider health/model extraction and ACP provider command facade.
- Create: `crates/agent-acp/src/session.rs` - provider-session command wrapper types.
- Add source-side test sidecars for: `crates/agent-acp/src/{content,events,permissions,provider,session}.rs` - serialization and conversion tests.
- Add fixture data under: `crates/agent-acp/src/fixtures/` - JSON inputs used by source-side ACP conversion tests.
- Modify: `crates/agent-protocol/src/lib.rs` - export the ACP protocol-neutral model module.
- Create: `crates/agent-protocol/src/acp.rs` - AgentCast-owned ACP session, event, content, permission, provider, and model types.
- Add source-side test sidecars for: `crates/agent-protocol/src/acp.rs` - protocol model serialization tests.

## Lab Decisions To Preserve

- ACP is provider-agnostic; public AgentCast types must not assume Codex.
- Event models must preserve raw `usage_update` and raw `ContentBlock[]`; text is a derived convenience, not the only stored representation.
- Unknown provider updates and unknown content block types must round-trip through raw JSON.
- Permission requests are explicit pause-and-resume events. There is no auto-approval path.
- Provider filesystem capabilities remain disabled or constrained until a workspace policy exists.
- Provider subprocesses must not inherit the full parent environment when runtime launch is implemented in `agent-runtime`.
- Action surfaces use one shared `action + params` semantic model, but that catalog is not owned by `agent-acp`.
- ACP should consume gateway tools through a narrow internal interface in post-v0 work; `agent-acp` must not become a second MCP gateway.

## Known Lab Fixes Already Reflected

The Lab ACP research findings identify issues that AgentCast must not reintroduce:

- do not flatten `ContentBlock[]` before persistence or API delivery.
- do not expose REST-shaped machine commands as the canonical semantic API when a shared dispatch model exists.
- do not replay only in-memory events when a persisted stream has older events.
- do not inherit all Lab or AgentCast credentials into provider subprocesses.
- do not auto-select the first permission option.
- do not place env reads inside reusable ACP adapter code.
- do not couple adapter errors to surface-layer error types.
- do not use `Bridge*` names for provider-agnostic AgentCast types.

## Implementation Tasks

### Task 1: Verify ACP SDK Patch Need

**Files:**
- Read: `Cargo.toml`
- Read: `../lab/Cargo.toml`
- Read: `../lab/crates/vendor/agent-client-protocol/Cargo.toml`
- Read: `../lab/crates/vendor/agent-client-protocol/src/session.rs`
- Modify only if required: `Cargo.toml`

- [ ] **Step 1: Check Lab's vendored patch rationale.**

Run:

```bash
sed -n '1,40p' ../lab/Cargo.toml
```

Expected: Lab documents a temporary `agent-client-protocol` patch because upstream `0.11.1` stripped `ActiveSession.models` in `attach_session()`.

- [ ] **Step 2: Check AgentCast's current ACP dependency.**

Run:

```bash
rg -n "agent-client-protocol" Cargo.toml crates/agent-acp/Cargo.toml crates/agent-gateway/Cargo.toml
```

Expected: AgentCast uses the workspace `agent-client-protocol` dependency and has no vendored patch by default.

- [ ] **Step 3: Decide the patch posture before coding.**

Expected: keep the upstream crate if model preservation is fixed in the installed version; add a workspace patch only if an AgentCast test proves model loss.

### Task 2: Define Protocol-Neutral ACP Models

**Files:**
- Modify: `crates/agent-protocol/src/lib.rs`
- Create: `crates/agent-protocol/src/acp.rs`
- Create: `crates/agent-protocol/src/acp.rs`

- [ ] **Step 1: Write serialization tests for ACP event ownership and raw preservation.**

Create a source-side test sidecar next to `crates/agent-protocol/src/acp.rs` with tests covering:

```rust
use super::*;
use serde_json::json;

#[test]
fn acp_message_chunk_round_trips_provider_owner() {
    let event = AcpEvent::MessageChunk {
        id: "evt-1".into(),
        created_at: "2026-05-05T00:00:00Z".into(),
        session_id: "session-1".into(),
        seq: 1,
        provider: "codex-acp".into(),
        role: "assistant".into(),
        text: "hello".into(),
        message_id: "msg-1".into(),
        content: vec![AcpContentBlock::Text { text: "hello".into() }],
    };

    let value = serde_json::to_value(&event).unwrap();
    assert_eq!(value["kind"], "message_chunk");
    assert_eq!(value["provider"], "codex-acp");
    assert_eq!(value["content"][0]["type"], "text");

    let decoded: AcpEvent = serde_json::from_value(value).unwrap();
    assert_eq!(decoded.provider_id(), Some("codex-acp"));
    assert_eq!(decoded.seq(), 1);
}

#[test]
fn acp_unknown_event_preserves_raw_payload() {
    let event = AcpEvent::Unknown {
        id: "evt-raw".into(),
        created_at: "2026-05-05T00:00:00Z".into(),
        session_id: "session-1".into(),
        seq: 7,
        event_kind: "provider_specific".into(),
        raw: json!({"nested": {"value": true}}),
    };

    let value = serde_json::to_value(&event).unwrap();
    assert_eq!(value["kind"], "unknown");
    assert_eq!(value["event_kind"], "provider_specific");
    assert_eq!(value["raw"]["nested"]["value"], true);
}

#[test]
fn acp_permission_option_uses_stable_kind_strings() {
    let option = AcpPermissionOption {
        option_id: "allow-once".into(),
        name: "Allow once".into(),
        kind: "allow_once".into(),
    };

    let value = serde_json::to_value(option).unwrap();
    assert_eq!(value["kind"], "allow_once");
}
```

- [ ] **Step 2: Run the protocol tests and confirm they fail before model implementation.**

Run:

```bash
cargo nextest run -p agent-protocol acp_
```

Expected: tests fail because `agent_protocol::acp` does not exist yet.

- [ ] **Step 3: Implement the protocol-neutral model module.**

Expected model families in `crates/agent-protocol/src/acp.rs`:

- `AcpSessionState`
- `AcpSessionSummary`
- `AcpModelOption`
- `AcpSessionConfigOptionView`
- `AcpProviderHealth`
- `AcpPermissionOption`
- `AcpContentBlock`
- `AcpEvent`

Expected event variants:

- `MessageChunk`
- `ReasoningChunk`
- `ToolCallStart`
- `ToolCallUpdate`
- `PermissionRequest`
- `PermissionOutcome`
- `UsageUpdate`
- `ContentBlocks`
- `SessionUpdate`
- `ProviderSwitch`
- `ProviderInfo`
- `Unknown`

- [ ] **Step 4: Export the module.**

Add this export in `crates/agent-protocol/src/lib.rs`:

```rust
pub mod acp;
```

- [ ] **Step 5: Re-run protocol tests.**

Run:

```bash
cargo nextest run -p agent-protocol acp_
```

Expected: protocol serialization tests pass.

### Task 3: Build ACP Adapter Module Skeleton

**Files:**
- Modify: `crates/agent-acp/src/lib.rs`
- Create: `crates/agent-acp/src/error.rs`
- Create: `crates/agent-acp/src/content.rs`
- Create: `crates/agent-acp/src/events.rs`
- Create: `crates/agent-acp/src/permissions.rs`
- Create: `crates/agent-acp/src/provider.rs`
- Create: `crates/agent-acp/src/session.rs`

- [ ] **Step 1: Replace the generated placeholder library.**

Expected `crates/agent-acp/src/lib.rs` exports:

```rust
pub mod content;
pub mod error;
pub mod events;
pub mod permissions;
pub mod provider;
pub mod session;

pub use error::{AcpAdapterError, Result};
```

- [ ] **Step 2: Add an adapter error type.**

Expected `crates/agent-acp/src/error.rs` responsibilities:

- represent ACP SDK serialization errors.
- represent unsupported or malformed provider updates.
- represent closed command channels.
- avoid depending on `agent-api`, `agent-cli`, or `agent-gateway`.

- [ ] **Step 3: Run focused crate tests.**

Run:

```bash
cargo nextest run -p agent-acp
```

Expected: the crate compiles after the generated `add` test is removed or replaced.

### Task 4: Extract Content And Attachment Conversion

**Files:**
- Modify: `crates/agent-acp/src/content.rs`
- Test sidecar: `crates/agent-acp/src/{content,events,permissions,provider,session}.rs`
- Source: `../lab/crates/lab/src/acp/runtime.rs`
- Source: `../lab/crates/lab/src/dispatch/acp/params.rs`

- [ ] **Step 1: Write tests for local attachment conversion.**

Cover:

- plain prompt text becomes an ACP text block.
- local text attachment becomes an embedded text resource.
- local blob attachment becomes an embedded blob resource.
- attachment names are percent-encoded in `file://local-attachment/...` URIs.
- attachment limits are enforced before conversion.

- [ ] **Step 2: Port the generic conversion behavior.**

Keep Lab's useful patterns:

- `prompt_input_to_content_blocks`
- `percent_encode_path_segment`
- local attachment count limit of 5
- local attachment byte limit of 48 KiB
- safe attachment name checks
- supported MIME checks

Rename/generalize:

- `LocalPromptAttachment` can remain a protocol input type if it is AgentCast-owned.
- `ToolError` must become `AcpAdapterError`.
- any Lab-specific parameter parsing belongs outside this crate.

- [ ] **Step 3: Verify conversion tests.**

Run:

```bash
cargo nextest run -p agent-acp attachment
```

Expected: conversion tests pass without launching a provider process.

### Task 5: Extract Permission Conversion

**Files:**
- Modify: `crates/agent-acp/src/permissions.rs`
- Test sidecar: `crates/agent-acp/src/{content,events,permissions,provider,session}.rs`
- Source: `../lab/crates/lab/src/acp/runtime.rs`

- [ ] **Step 1: Write tests for permission option conversion.**

Cover ACP SDK option kinds:

- allow once -> `allow_once`
- allow always -> `allow_always`
- reject once -> `reject_once`
- reject always -> `reject_always`
- unknown -> `unknown`

- [ ] **Step 2: Extract conversion helpers.**

Keep conversion behavior from Lab's `acp_permission_option_from_protocol`, but remove runtime ownership of pending permission maps.

- [ ] **Step 3: Verify permission tests.**

Run:

```bash
cargo nextest run -p agent-acp permission
```

Expected: permission conversion tests pass.

### Task 6: Extract Event Normalization

**Files:**
- Modify: `crates/agent-acp/src/events.rs`
- Test sidecar: `crates/agent-acp/src/{content,events,permissions,provider,session}.rs`
- Source: `../lab/crates/lab/src/acp/runtime.rs`
- Source: `../lab/crates/lab-apis/src/acp/types.rs`

- [ ] **Step 1: Write tests for provider update normalization.**

Cover:

- user message chunks and assistant message chunks keep stable message IDs.
- reasoning chunks become `ReasoningChunk`.
- tool calls become `ToolCallStart` plus provider info when metadata exists.
- tool call updates preserve merged `_meta`.
- usage updates become `UsageUpdate` with raw JSON.
- unknown updates become `ProviderInfo` or `Unknown` with raw JSON.
- content blocks preserve raw block detail and derive text only as convenience.

- [ ] **Step 2: Port normalization helpers without runtime loops.**

Keep:

- `tool_call_update_output`
- `enum_value`
- `map_stop_reason`
- provider info wrapping
- raw `_meta` preservation without logging its values.

Change:

- do not use Lab's `content_to_text` as the only representation.
- return AgentCast `AcpEvent` values instead of sending directly to an event channel.
- accept timestamps and event IDs from caller or a small injected generator so tests remain deterministic.

- [ ] **Step 3: Verify event tests.**

Run:

```bash
cargo nextest run -p agent-acp event
```

Expected: normalization tests pass without Codex, Node.js, or subprocess fixtures.

### Task 7: Extract Provider Health And Model Metadata

**Files:**
- Modify: `crates/agent-acp/src/provider.rs`
- Test sidecar: `crates/agent-acp/src/{content,events,permissions,provider,session}.rs`
- Source: `../lab/crates/lab/src/acp/runtime.rs`
- Source: `../lab/crates/lab/src/acp/providers.rs`

- [ ] **Step 1: Write tests for model extraction.**

Cover:

- provider session models populate available model options.
- current model ID is preserved.
- fixed/default model flags remain explicit.
- missing model metadata produces an empty model list, not an error.

- [ ] **Step 2: Extract model conversion.**

Keep Lab's `session_model_options` behavior, but make the output AgentCast protocol models.

- [ ] **Step 3: Define provider entry shape only if needed by adapter tests.**

If needed, keep only the generic structured provider command shape:

- `id`
- `name`
- `version`
- `distribution`
- `command`
- `args`
- `cwd`
- `env`
- `installed_at`
- `sha256`

Provider file I/O and install storage stay outside `agent-acp`.

- [ ] **Step 4: Verify provider tests.**

Run:

```bash
cargo nextest run -p agent-acp provider
```

Expected: provider metadata tests pass.

### Task 8: Define Provider Session Command Facade

**Files:**
- Modify: `crates/agent-acp/src/session.rs`
- Test sidecar: `crates/agent-acp/src/{content,events,permissions,provider,session}.rs`
- Source: `../lab/crates/lab-apis/src/acp/session.rs`
- Source: `../lab/crates/lab/src/acp/runtime.rs`

- [ ] **Step 1: Write bounded command-channel tests.**

Cover:

- prompt commands can be sent.
- cancel commands can be sent.
- channel closure maps to `AcpAdapterError`.
- queue saturation has a deterministic error path.

- [ ] **Step 2: Extract only the bounded session command facade.**

Keep:

- bounded `mpsc::Sender` semantics.
- explicit `Prompt` and `Cancel` command variants.

Do not include:

- per-session OS thread creation.
- provider process launch.
- runtime shutdown timeout.
- prompt idle timeout.
- turn drain timeout.
- permission pending map ownership.

- [ ] **Step 3: Verify session tests.**

Run:

```bash
cargo nextest run -p agent-acp session
```

Expected: session facade tests pass without external processes.

### Task 9: Record Hand-Offs To Other Crates

**Files:**
- Modify: `docs/plans/extract-crates/agent-runtime.md`
- Modify: `docs/plans/extract-crates/agent-gateway.md`
- Modify: `docs/plans/extract-crates/agent-api.md`
- Modify: `docs/plans/extract-crates/agent-cli.md`
- Modify: `docs/plans/extract-crates/agent-marketplace.md`
- Modify: `docs/plans/extract-crates/agent-registry.md`

- [ ] **Step 1: Add explicit runtime hand-off notes.**

Expected `agent-runtime` owns:

- session registry.
- principal checks.
- event sequence stamping and replay.
- persistent event storage.
- provider launch/supervision.
- turn drain and idle completion.
- permission pending maps and timeout decisions.

- [ ] **Step 2: Add explicit gateway/surface hand-off notes.**

Expected `agent-gateway`, `agent-api`, and `agent-cli` own:

- `action + params` catalog projection.
- destructive confirmation gates.
- API route mapping.
- CLI command syntax.
- MCP tool exposure.

- [ ] **Step 3: Add explicit registry/marketplace hand-off notes.**

Expected `agent-registry` and `agent-marketplace` own:

- ACP Registry discovery.
- provider package metadata.
- provider install-plan preview/apply.
- install conflict behavior.

## Verification Commands

Run these commands after implementation:

```bash
cargo nextest run -p agent-protocol acp_
cargo nextest run -p agent-acp
cargo nextest run -p agent-runtime acp
```

Expected: `agent-protocol` and `agent-acp` tests pass; `agent-runtime acp` is required only once runtime-owned ACP session work starts.

## Contract And Spec Follow-Ups

Create these docs when the implementation begins:

- `docs/contracts/acp-events.md` - stable event envelope, sequence, raw-preservation, and unknown-event rules.
- `docs/contracts/acp-permissions.md` - permission request/outcome invariants and no-auto-approval rule.
- `docs/specs/acp-adapter.md` - crate-level ACP adapter implementation details.
- `docs/specs/acp-runtime.md` - runtime-owned session lifecycle, provider launch, persistence, and replay details.

## Self-Review

- Spec coverage: covers the requested `agent-acp` extraction and names the Lab sources inspected.
- Boundary coverage: separates adapter code from runtime, persistence, registry, marketplace, API, CLI, MCP, and UI work.
- Placeholder scan: the plan contains concrete paths, commands, and expected outcomes instead of empty implementation steps.
- Type consistency: `Acp*` names are used for provider-agnostic AgentCast models; Lab `Bridge*` names are excluded from new public types.
