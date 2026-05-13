---
title: "Reference Refresh And Axon Filename Stability"
doc_type: "session"
status: "historical"
owner: "agentcast"
audience:
  - "maintainers"
  - "contributors"
scope: "historical"
source_of_truth: false
upstream_refs:
  - "docs/references/CHANGES-REPORT.md"
  - "docs/references/CHANGES.md"
  - "docs/references/INDEX.md"
  - "docs/references/acp/docs/manifest.jsonl"
  - "docs/references/claude-code/manifest.jsonl"
  - "docs/references/mcp/docs/manifest.jsonl"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "historical note reviewed against current local docs/references snapshot"
date: "2026-05-12 14:35:21 EDT"
repo: "git@github.com:jmagar/agentcast.git"
branch: "main"
head: "unavailable - repository has no current commit"
agent: "Codex"
session_id: "d4865594-27ec-47e2-a666-31abdcd815de"
transcript: "/home/jmagar/.claude/projects/-home-jmagar-workspace-agentcast/d4865594-27ec-47e2-a666-31abdcd815de.jsonl"
working_directory: "/home/jmagar/workspace/agentcast"
worktree: "/home/jmagar/workspace/agentcast  0000000 [main]"
historical: true
---

# Reference Refresh And Axon Filename Stability

## User Request

Create and test a reference-doc refresh workflow for Agentcast, then diagnose why the generated reference change report showed hundreds of added and removed crawled docs.

## Session Overview

- Built a `refresh-docs` skill under `.agents/skills/refresh-docs`.
- Created `scripts/refresh-docs.sh` to refresh Axon-crawled docs, Repomix XML packs, sparse-cloned docs, `INDEX.md`, and append-only `CHANGES.md`.
- Created `scripts/review-changes.sh` to invoke Claude Code headless mode and run the local `refresh-docs` skill.
- Ran the full Claude flow with streaming enabled.
- Found that crawled-doc change summaries were dominated by filename churn, not real content changes.
- Traced the filename churn to Axon's order-indexed markdown filenames in `../axon_rust`.

Scope E audit note, 2026-05-13: this is a historical session record. The current reference
inventory should be read from `docs/references/INDEX.md` and the manifests under
`docs/references/acp/docs/manifest.jsonl`, `docs/references/mcp/docs/manifest.jsonl`, and
`docs/references/claude-code/manifest.jsonl`. The Axon filename-stability finding remains a
workflow note, not an upstream protocol claim.

## Sequence Of Events

- Created local reference layout under `docs/references/` for ACP, MCP, Claude Code, FastMCP, mcporter, and jmagar references.
- Added `.agents/skills/refresh-docs` as the source of truth for local development skills and linked `.claude/skills` to `.agents/skills`.
- Implemented report archiving with `.agents/skills/refresh-docs/scripts/archive-changes-report.sh`.
- Added streaming to `scripts/review-changes.sh` using `claude -p --output-format stream-json --verbose --include-partial-messages --include-hook-events`.
- Ran the full review flow and generated `docs/references/CHANGES-REPORT.md`.
- Reviewed the report and found it overclaimed that `modified = 0` proved no content changes.
- Investigated Axon and found crawl markdown filenames are generated with a crawl-order index.

## Key Findings

- `scripts/review-changes.sh` streams successfully and invokes the `refresh-docs` skill.
- `docs/references/CHANGES.md` records path-level changes, not semantic document identity changes.
- Axon crawl docs use filenames like `0001-...md`; the numeric prefix comes from the current successful markdown write count.
- Axon's `url_to_filename(url, idx)` is defined in `../axon_rust/src/core/content.rs`.
- Axon crawl collection calls `url_to_filename(url, summary.markdown_files + 1)` through `process_page`, so the same URL can get a different filename when crawl order changes.

## Technical Decisions

- The Agentcast refresh script should not treat Axon crawl filename churn as real document additions/removals.
- The better fix is in Axon: make markdown filenames stable by URL identity, not crawl discovery order.
- Keep human-readable slugs, but add a URL hash to avoid collisions and truncation ambiguity.
- Existing order-indexed `url_to_filename(url, idx)` may remain for single-document outputs if needed, but crawl outputs should use a stable filename function.

## Files Modified

- `.agents/skills/refresh-docs/SKILL.md`: local skill for running refresh, inspecting `CHANGES.md`, and generating `CHANGES-REPORT.md`.
- `.agents/skills/refresh-docs/scripts/archive-changes-report.sh`: archives an existing active report before writing a new one.
- `.agents/skills/refresh-docs/references/CHANGES-template.md`: template/reference for the append-only change log.
- `.agents/skills/refresh-docs/references/CHANGES-REPORT-template.md`: report structure for review output.
- `scripts/refresh-docs.sh`: refreshes all reference material and appends `docs/references/CHANGES.md`.
- `scripts/review-changes.sh`: terminal entry point that invokes Claude Code headless mode.
- `docs/references/CHANGES.md`: append-only log of reference refresh runs.
- `docs/references/CHANGES-REPORT.md`: active report generated by the corrected flow.

## Commands Executed

- `scripts/review-changes.sh --max-turns 40`: ran the end-to-end Claude headless review workflow.
- `bash -n scripts/refresh-docs.sh scripts/review-changes.sh`: validated shell syntax.
- `rg -n '^## [0-9]|summary:' docs/references/CHANGES.md`: inspected refresh entries.
- `sed -n '1,240p' docs/references/CHANGES-REPORT.md`: reviewed the generated impact report.
- `rg -n "fn url_to_filename|url_to_filename\\(" src/core src/crawl -S` in `../axon_rust`: found Axon filename generation call sites.

## Errors Encountered

- The first Claude run reviewed the first `CHANGES.md` entry instead of the latest appended entry. Fixed the skill's dynamic awk block to read the last entry.
- A verification command accidentally invoked `archive-changes-report.sh --help` before the helper supported `--help`, archiving the active report. Restored the report and added safe help handling.
- The report overclaimed that `modified = 0` proved no content changes. The actual issue is path churn from unstable filenames.

## Behavior Changes

- Before: `review-changes.sh` could run silently in text mode and the skill could review the wrong `CHANGES.md` entry.
- After: `review-changes.sh` streams Claude events by default, and the skill reads the latest appended change entry.
- Remaining issue: `CHANGES.md` still receives noisy added/removed entries when Axon crawl order changes.

## Verification Evidence

| Command | Expected | Actual | Status |
| --- | --- | --- | --- |
| `scripts/review-changes.sh --max-turns 40` | Full headless Claude flow runs | Completed, streamed, archived prior report, wrote `CHANGES-REPORT.md` | Pass |
| `bash -n scripts/refresh-docs.sh scripts/review-changes.sh` | No syntax errors | No output, exit 0 | Pass |
| `rg -n '^## [0-9]|summary:' docs/references/CHANGES.md` | Latest entry visible | Latest entry was `2026-05-12T18:12:17Z` | Pass |
| `rg -n "url_to_filename" ../axon_rust/src` | Find Axon filename generator and call sites | Found order-indexed crawl, sitemap, thin-refetch, and Chrome call sites | Pass |

## Risks And Rollback

- Changing Axon filenames may affect consumers that rely on current `0001-...md` paths.
- Rollback is to restore order-indexed `url_to_filename(url, idx)` use in crawl output paths.
- Compatibility can be preserved by keeping `relative_path` in `manifest.jsonl` and avoiding assumptions about numeric order.

## Decisions Not Taken

- Did not patch Agentcast `refresh-docs.sh` to normalize Axon filenames yet, because the better root fix is in Axon.
- Did not create fake drift fixtures after the user objected; the full flow was tested with real refresh output instead.

## Open Questions

- Whether Axon should preserve old filenames for unchanged URLs by reading `previous_manifest`, or migrate all crawl outputs to a URL-hash filename immediately.
- Whether manifest entries should be sorted by URL before writing to reduce diff noise further.
- Whether Axon should provide an explicit stable export mode for documentation mirroring workflows.

## Next Steps

- Patch `../axon_rust` so crawled markdown filenames are URL-stable.
- Add regression tests proving the same URL receives the same filename independent of crawl order.
- Consider updating Agentcast `refresh-docs.sh` later to compare crawled docs by manifest URL and content hash for stronger reports.
