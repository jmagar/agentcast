---
title: "Reference Changes Impact Report"
doc_type: "report"
status: "generated"
owner: "refresh-docs"
audience:
  - "maintainers"
  - "contributors"
scope: "reference"
source_of_truth: false
upstream_refs:
  - "docs/references/CHANGES.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "reference artifact reviewed against local docs/references manifests and refresh logs"
generated_at: "2026-05-12T18:15:00Z"
source_changes_log: "docs/references/CHANGES.md"
reviewed_change_entry: "2026-05-12T18:12:17Z"
reviewer: "refresh-docs skill"
---

# Reference Changes Impact Report

## Latest Refresh Summary

Entry timestamp: `2026-05-12T18:12:17Z`

- Script: `scripts/refresh-docs.sh`
- Scope: full
- Axon output dir: `/home/jmagar/.axon/output`
- Summary: `377 added, 0 modified, 377 removed`

This is the fourth and latest refresh entry in `docs/references/CHANGES.md` today. Three prior
runs occurred at 17:51:24Z, 17:58:28Z, and 18:02:42Z. The latest entry supersedes all prior
entries as the canonical reference state.

The 377 added and 377 removed files are a pure renumbering event. Modified = 0 confirms
zero content changes. Every removed file has a slug-identical counterpart in the added list;
only the numeric filename prefix changed.

File counts per domain in this entry:

| Domain | Added | Removed | On disk (total, incl. carry-overs) |
|---|---|---|---|
| `acp/docs/markdown/` | 71 | 71 | 73 |
| `claude-code/markdown/` | 122 | 122 | 132 |
| `mcp/docs/markdown/` | 184 | 184 | 188 |
| **Total tracked** | **377** | **377** | **393** |

The 16 files on disk that do not appear in this entry's added list are carry-overs from earlier
refresh cycles. They are genuine reference files; the latest `CHANGES.md` path-diff did not list
them under added paths because those paths were already present. See [Unknowns](#unknowns) for
details.

Scope E audit correction, 2026-05-13: the current manifests were rechecked against the filesystem
and contain all 393 markdown files with no missing or extra markdown paths
(`acp/docs`: 73, `claude-code`: 132, `mcp/docs`: 188). The 16 carry-over files also appear in
their current manifests. Treat this report as an impact review of the latest `CHANGES.md` path-diff
entry, not as a complete manifest inventory.

## Changed References Reviewed

All 377 added files were matched against the 377 removed files by slug. Every slug matched
exactly, confirming pure renumbering with no content loss and no new content introduced.
The review below is organized by directory.

Scope E audit correction, 2026-05-13: slug matching confirms that every added path has a
slug-identical removed path. The latest `CHANGES.md` entry by itself does not include enough
content-hash data to independently prove byte-for-byte content equality for removed files.

### `docs/references/acp/docs/markdown/` — 71 tracked files

Status: **renumbered, content unchanged**.

Covers the full public ACP documentation corpus. Key content areas confirmed present:

- **Protocol core**: `initialization`, `overview`, `session-setup`, `session-modes`,
  `session-config-options`, `session-list`, `prompt-turn`, `content`, `tool-calls`,
  `file-system`, `terminals`, `slash-commands`, `agent-plan`, `extensibility`,
  `transports`, `schema`
- **Get started**: `introduction`, `architecture`, `agents`, `clients`, `registry`
- **Libraries**: `rust`, `typescript`, `python`, `kotlin`, `java`, `community`
- **Stabilized announcements**: `session-resume`, `session-list`, `session-close`,
  `session-config-options`, `session-info-update`, `acp-agent-registry`
- **RFDs (drafts/proposals)**: `v2-overview`, `v2-prompt`, `proxy-chains`, `mcp-over-acp`,
  `rust-sdk-v1`, `session-fork`, `session-close`, `session-delete`, `session-list`,
  `session-resume`, `session-info-update`, `session-config-options`, `session-usage`,
  `elicitation`, `request-cancellation`, `streamable-http-websocket-transport`,
  `message-id`, `agent-telemetry-export`, `acp-agent-registry`, `next-edit-suggestions`,
  `additional-directories`, `diff-delete`, `boolean-config-option`, `model-config-category`,
  `meta-propagation`, `updates`, `custom-llm-endpoint`

### `docs/references/claude-code/markdown/` — 122 tracked files

Status: **renumbered, content unchanged**.

Covers the full Claude Code and Agent SDK documentation. Key content areas confirmed present:

- **Agent SDK**: `overview`, `quickstart`, `python`, `typescript`, `typescript-v2-preview`,
  `agent-loop`, `sessions`, `subagents`, `hooks`, `hooks-guide`, `permissions`,
  `custom-tools`, `user-input`, `streaming-output`, `streaming-vs-single-mode`, `mcp`,
  `skills`, `slash-commands`, `plugins`, `observability`, `cost-tracking`, `todo-tracking`,
  `modifying-system-prompts`, `claude-code-features`, `file-checkpointing`, `tool-search`,
  `secure-deployment`, `hosting`, `migration-guide`, `structured-outputs`
- **Platform docs**: `overview`, `quickstart`, `desktop-quickstart`, `web-quickstart`,
  `setup`, `authentication`, `permissions`, `permission-modes`, `settings`, `env-vars`,
  `memory`, `hooks`, `mcp`, `agents`, `sub-agents`, `agent-teams`, `features-overview`,
  `keybindings`, `plugins`, `plugins-reference`, `plugin-dependencies`, `plugin-marketplaces`,
  `skills`, `commands`, changelogs, cloud/CI guides, weekly whats-new entries W13–W18 2026

### `docs/references/mcp/docs/markdown/` — 184 tracked files

Status: **renumbered, content unchanged**.

Covers the full MCP specification corpus including all historical versions
(2024-11-05, 2025-03-26, 2025-06-18, 2025-11-25, draft), SEPs, registry docs,
community docs, extension overviews, and tutorial content.

## Affected Implementation Areas

No implementation areas are affected by this refresh. The content of all reference files
is identical to the prior state; only numeric filename prefixes changed. Additionally, every
crate in `crates/` is currently an unimplemented stub (`pub fn add(left: u64, right: u64) -> u64`).
There is no implementation surface to align.

The following areas will be relevant once implementation begins:

| Area | Crate(s) | Key reference files |
|---|---|---|
| ACP session lifecycle | `agentcast-acp`, `agentcast-protocol` | `acp/docs/markdown/0011-*initialization`, `0045-*session-setup`, `0062-*prompt-turn`, `0063-*session-list`, `0061-*session-config-options` |
| ACP agent capabilities | `agentcast-acp` | `0009-*rfds-session-resume`, `0070-*rfds-session-fork`, `0034-*rfds-session-close`, `0023-*rfds-acp-agent-registry` |
| MCP integration | `agentcast-mcp`, `agentcast-gateway` | `mcp/docs/markdown/` spec files for the 2025-03-26 and 2025-06-18 versions |
| Agent registry | `agentcast-registry`, `agentcast-marketplace` | `acp/docs/markdown/0043-*get-started-registry`, `0023-*rfds-acp-agent-registry` |
| Claude Agent SDK hosting | `agentcast-server`, `agentcast-runtime` | `claude-code/markdown/0005-*agent-sdk-hosting`, `0006-*agent-sdk-agent-loop` |
| Config options / session modes | `agentcast-acp` | `0061-*session-config-options`, `0008-*session-modes` (modes deprecated in favor of configOptions) |

## Required Changes

None. The reference content did not change, and the implementation is entirely stubbed out.
No existing code contradicts or lags behind the refreshed references.

## New Additions To Consider

The references do not introduce net-new content vs. the prior state (pure renumber), but
the full corpus now clearly indexed in `docs/references/` documents several features that
should shape initial implementation work when development begins:

**ACP stabilized features (all stabilized as of 2026):**

1. `session/resume` — allows reconnecting to an existing session without replaying history.
   Capability advertisement: `sessionCapabilities.resume: {}`. Supersedes some `session/load`
   use cases. Source: `acp/docs/markdown/0009-*rfds-session-resume`,
   `acp/docs/markdown/0010-*announcements-session-resume-stabilized`.

2. `session/list` — cursor-paginated listing of sessions known to an agent, with optional
   `cwd` filter. Capability: `sessionCapabilities.list: {}`. Includes real-time
   `session_info_update` notifications. Source: `acp/docs/markdown/0063-*session-list`.

3. `session/close` — instructs the agent to cancel ongoing work and free resources.
   Capability: `sessionCapabilities.close: {}`. Source: `acp/docs/markdown/0045-*session-setup`
   (Closing Active Sessions section), `0034-*rfds-session-close`.

4. Session Config Options — replaces session modes for model/mode/reasoning-level selectors.
   Clients should use `configOptions` (with `category: "mode"`) instead of the `modes` field;
   `modes` is deprecated and will be removed. During transition, both should be emitted in sync.
   Source: `acp/docs/markdown/0061-*session-config-options`, `0071-*session-config-options-stabilized`.

5. ACP Agent Registry — canonical manifest format (`id/agent.json`) and distribution spec
   (`binary`, `npx`, `uvx`). The `agentcast-registry` and `agentcast-marketplace` crates
   should implement this. Source: `acp/docs/markdown/0023-*rfds-acp-agent-registry`,
   `0043-*get-started-registry`.

**ACP draft/proposed features (not yet stabilized — design input only):**

- `session/fork` (RFD, not stabilized) — fork a session to run side tasks without polluting
  history. Relevant for `agentcast-acp`. Source: `0070-*rfds-session-fork`.
- Configurable LLM providers (`providers/list`, `providers/set`, `providers/disable`) (RFD) —
  lets clients route LLM traffic through proxies or gateways. Source: `0017-*rfds-custom-llm-endpoint`.
- Agent telemetry export (RFD) — standard telemetry surface. Source: `0064-*rfds-agent-telemetry-export`.

**Claude Agent SDK features (for `agentcast-runtime`, `agentcast-server`):**

- Session management patterns: `ClaudeSDKClient` / `continue: true` / `resume` / `fork`.
  Source: `claude-code/markdown/0024-*agent-sdk-sessions`.
- Subagents: programmatic `AgentDefinition` with tool restrictions and specialized prompts.
  Source: `claude-code/markdown/0026-*agent-sdk-subagents`.
- Structured outputs, streaming output, tool search: `0018-*structured-outputs`,
  `0019-*streaming-output`, `0014-*tool-search`.
- Skills integration via `settingSources` / `skills` option.
  Source: `claude-code/markdown/0022-*agent-sdk-skills`.
- Hosting / secure deployment patterns for containerized agents.
  Source: `0005-*agent-sdk-hosting`, `0055-*agent-sdk-secure-deployment`.

**MCP spec features (for `agentcast-mcp`, `agentcast-gateway`):**

- The `rmcp` crate (v1.6) is already declared in `Cargo.toml` with `elicitation`, `auth`,
  `transport-streamable-http-server`, and `transport-streamable-http-client-reqwest` features.
  MCP spec coverage for elicitation and streamable HTTP is available in
  `mcp/docs/markdown/0082-*draft-client-elicitation` and related transport spec files.
- MCP Apps (interactive UI widgets) — SEP 1865 / extensions-apps docs. Potentially
  relevant to `agentcast-gateway` for surfacing agent UI.
  Source: `mcp/docs/markdown/0077-*extensions-apps-overview`, `0137-*extensions-apps-build`.

## Verification To Run

Since no implementation code exists yet, there is no regression testing to run against the
refreshed references. When implementation begins, the following should be verified:

1. **ACP initialization handshake**: test that `initialize` round-trips protocol version 1,
   all declared capabilities (`sessionCapabilities.resume`, `sessionCapabilities.list`,
   `sessionCapabilities.close`, `mcpCapabilities.http`, `promptCapabilities.*`), and
   `agentInfo` fields. Reference: `acp/docs/markdown/0011-*initialization`.

2. **Session lifecycle round-trips**: `session/new` → `session/prompt` → `session/cancel`
   for baseline; then `session/resume`, `session/list`, `session/close` behind capability checks.

3. **Config options vs. modes parity**: if `configOptions` with `category: "mode"` is
   implemented, ensure both `configOptions` and the legacy `modes` field are emitted in sync
   during the deprecation period.

4. **ACP agent registry manifest schema**: validate any `agent.json` manifest generated by
   `agentcast-registry` against the id-pattern, required fields, and distribution format
   defined in `0023-*rfds-acp-agent-registry`.

5. **MCP `rmcp` feature flags**: confirm that the features enabled in `Cargo.toml`
   (`elicitation`, `auth`, `transport-streamable-http-server`, etc.) align with the ACP
   capability advertisements in `agentcast-acp`. Cross-check with ACP session-setup docs
   (`mcpCapabilities.http`).

6. **Agent SDK hosted session management**: if `agentcast-server` wraps the Claude Agent SDK,
   verify session ID tracking (resume vs. continue vs. fork) matches the patterns documented
   in `claude-code/markdown/0024-*agent-sdk-sessions`.

## Unknowns

1. **16 carry-over files not tracked in the latest CHANGES.md entry.** The filesystem
   contains 393 files across the three reference directories, but the latest entry only
   accounts for 377 (71 + 122 + 184). The additional 16 files
   (`0059-*schema.md`, `0073-*.md` in ACP; 10 claude-code files including `0082-*model-config`,
   `0109-*sub-agents`, `0127-*whats-new-2026-w19`, `0130-*whats-new`, `0131-*zero-data-retention`, etc.;
   4 MCP files including `0080-*2025-06-18-client-sampling`, `0186-*draft-schema`) were
   present from earlier refresh cycles and were not re-emitted as new in this run.
   These files are genuine reference content and were reviewed as part of the overall corpus
   above. They are also present in the current manifests. Their presence relative to the
   CHANGES.md path-diff entry remains an audit gap.
   Recommendation: future reports should compare the latest path-diff entry against the current
   manifests so unchanged carry-over paths are explicitly accounted for.

2. **`session/fork` RFD status.** The RFD (`0070-*rfds-session-fork`) was last revised
   2025-12-10 and does not appear in the stabilized announcements list. Its canonical
   status (draft, accepted, rejected) cannot be determined from the file alone. Confirm
   with the ACP upstream whether this capability should be advertised in the `initialize`
   response before implementing it in `agentcast-acp`.

3. **ACP `session/delete` vs. `session/close` distinction.** Both RFDs are present
   (`0026-*rfds-session-delete`, `0034-*rfds-session-close`). The stabilized announcement
   only covers `session/close`. Whether `session/delete` (deleting from history, not just
   closing an active session) is relevant to `agentcast-acp` should be reviewed when
   implementing session lifecycle.

4. **Configurable LLM providers RFD status.** The `providers/list`, `providers/set`,
   `providers/disable` RFD (`0017-*rfds-custom-llm-endpoint`) has no corresponding stabilized
   announcement in the corpus. Whether to implement this in `agentcast-config` or
   `agentcast-gateway` depends on product scope, not just protocol availability.
