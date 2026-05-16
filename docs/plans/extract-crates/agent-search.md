---
title: "agent-search Extraction Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md"
  - "docs/references/mcp/docs/markdown/0185-modelcontextprotocol-io-docs-learn-architecture.md"
related: []
last_reviewed: "2026-05-15"
last_modified: "2026-05-15"
modified_on_branch: "gateway-first-skeleton"
modified_at_version: "0.1.0"
modified_at_commit: "d327495"
review_basis: "cross-referenced against gateway-first implementation audit and local docs/references snapshot"
---

# agent-search Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `agent-search` as the launcher/catalog search and ranking crate for AgentCast.

**Architecture:** Search indexes actions and metadata produced by runtime, gateway, registry, and marketplace crates. It must not own invocation, routing, registry fetches, or UI rendering.

**Tech Stack:** Rust 2024, serde/serde_json, thiserror, tracing.

---

## MVP Position

`agent-search` can support the v0 launcher if action discovery needs ranking beyond a simple list. Keep the first implementation deterministic and explainable.

## Current Implementation Audit

As of 2026-05-15, `agent-search` is partially implemented for the gateway path with deterministic action documents, indexing, query matching, ranked results used by the gateway API, secret-like field redaction, bounded/truncated field normalization, schema-summary indexing, and stable result explanation metadata for matched terms and fields.

Continue only when additional ranking signals or UI/API consumers require broader query behavior.

## Lab Evidence Read

- `../lab/crates/lab/src/dispatch/gateway/index.rs`
- `../lab/crates/lab/src/dispatch/gateway/projection.rs`
- `../lab/crates/lab/src/dispatch/gateway/runtime.rs`
- `../lab/crates/lab/src/config.rs`
- `../lab/apps/gateway-admin/lib/app-command-palette.ts`
- `../lab/apps/gateway-admin/components/design-system/command-palette-model.ts`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab search source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`; the packed snapshot does not include `crates/lab/src/dispatch/gateway/manager.rs`, so search evidence must come from `index.rs`, `projection.rs`, `runtime.rs`, and `config.rs`.
- MCP discovery/listing assumptions are cross-checked against `docs/references/mcp/docs/markdown/0185-modelcontextprotocol-io-docs-learn-architecture.md`, `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`, `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`, and `docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md`.

Live source discovery command:

```bash
rg -n "search|rank|catalog|alias|category|filter|fuzzy|tool search" ../lab
```

## Live Lab Findings

- `dispatch/gateway/index.rs` has the core Rust search implementation: indexed tool documents, metadata, search hits, scoring, and catalog hashing.
- Gateway call sites invoke `search_tools`, but the reusable ranking logic is in `index.rs`; refresh/rebuild orchestration should remain gateway/runtime behavior, not search ranking logic.
- Admin UI command palette code is useful for future UI search expectations, not v0 Rust behavior.

## Extraction Boundary

Extract into `agent-search`:

- normalized searchable document shapes for launcher actions.
- tokenization and alias/category matching.
- deterministic ranking rules.
- future recency/favorites signal interfaces.
- search-result explanation metadata for CLI/API/UI.

Keep out of `agent-search`:

- MCP tool invocation.
- MCP server lifecycle.
- registry network fetches.
- gateway collision decisions.
- CLI/API/UI rendering.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-search/src/lib.rs` - public exports for documents, index, query, and results.
- Create: `crates/agent-search/src/document.rs` - searchable action document model.
- Create: `crates/agent-search/src/index.rs` - deterministic in-memory search index.
- Create: `crates/agent-search/src/query.rs` - query normalization and limits.
- Add source-side test sidecars for: `crates/agent-search/src/index.rs` - deterministic ranking tests.
- Add source-side test sidecars for: `crates/agent-search/src/index.rs` - empty-query behavior tests.

## Implementation Tasks

### Task 1: Port Deterministic Ranking Fixtures

**Files:**
- Create: `crates/agent-search/src/index.rs`
- Test sidecar: `crates/agent-search/src/index.rs`

- [ ] **Step 1: Read Lab scoring and hashing.**

Run:

```bash
sed -n '1,180p' ../lab/crates/lab/src/dispatch/gateway/index.rs
```

Expected: AgentCast ranking tests cover name, description, upstream, aliases/categories once added, top-k truncation, empty query behavior, and stable catalog hashes.

- [ ] **Step 2: Write failing ranking tests.**

Create a source-side test sidecar next to `crates/agent-search/src/index.rs` with:

```rust
use super::*;

#[test]
fn exact_title_match_ranks_before_description_match() {
    let index = SearchIndex::new([
        SearchDocument::new("mcp:git:tool:status", "Git status").description("Show repository status"),
        SearchDocument::new("mcp:filesystem:tool:read_file", "Read file").description("Read git config files"),
    ]);

    let results = index.search(SearchQuery::new("git status").limit(10));
    assert_eq!(results[0].action_id, "mcp:git:tool:status");
}

#[test]
fn search_honors_limit() {
    let index = SearchIndex::new([
        SearchDocument::new("a", "match one"),
        SearchDocument::new("b", "match two"),
    ]);

    assert_eq!(index.search(SearchQuery::new("match").limit(1)).len(), 1);
}
```

- [ ] **Step 3: Export search modules.**

Update `crates/agent-search/src/lib.rs`:

```rust
mod document;
mod index;
mod query;

pub use document::SearchDocument;
pub use index::{SearchIndex, SearchResult};
pub use query::SearchQuery;
```

- [ ] **Step 4: Implement documents and query.**

Create `crates/agent-search/src/document.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchDocument {
    pub action_id: String,
    pub title: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub categories: Vec<String>,
}

impl SearchDocument {
    pub fn new(action_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            title: title.into(),
            description: String::new(),
            aliases: Vec::new(),
            categories: Vec::new(),
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}
```

Create `crates/agent-search/src/query.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    pub text: String,
    pub limit: usize,
}

impl SearchQuery {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            limit: 20,
        }
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn terms(&self) -> Vec<String> {
        self.text
            .split_whitespace()
            .map(|term| term.to_ascii_lowercase())
            .collect()
    }
}
```

- [ ] **Step 5: Implement deterministic ranking.**

Create `crates/agent-search/src/index.rs`:

```rust
use crate::{SearchDocument, SearchQuery};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub action_id: String,
    pub score: i64,
    pub explanation: String,
}

#[derive(Debug, Clone)]
pub struct SearchIndex {
    documents: Vec<SearchDocument>,
}

impl SearchIndex {
    pub fn new(documents: impl IntoIterator<Item = SearchDocument>) -> Self {
        Self {
            documents: documents.into_iter().collect(),
        }
    }

    pub fn search(&self, query: SearchQuery) -> Vec<SearchResult> {
        let terms = query.terms();
        if terms.is_empty() {
            return Vec::new();
        }

        let mut results = self
            .documents
            .iter()
            .filter_map(|doc| score_document(doc, &terms))
            .collect::<Vec<_>>();
        results.sort_by(|left, right| right.score.cmp(&left.score).then(left.action_id.cmp(&right.action_id)));
        results.truncate(query.limit);
        results
    }
}

fn score_document(doc: &SearchDocument, terms: &[String]) -> Option<SearchResult> {
    let title = doc.title.to_ascii_lowercase();
    let description = doc.description.to_ascii_lowercase();
    let mut score = 0;
    for term in terms {
        if title.contains(term) {
            score += 10;
        }
        if description.contains(term) {
            score += 2;
        }
    }
    (score > 0).then(|| SearchResult {
        action_id: doc.action_id.clone(),
        score,
        explanation: format!("matched {} term(s)", terms.len()),
    })
}
```

### Task 2: Define Empty Query Behavior

**Files:**
- Test sidecar: `crates/agent-search/src/index.rs`
- Modify: `crates/agent-search/src/index.rs`

- [ ] **Step 1: Write empty-query test.**

Create a source-side test sidecar next to `crates/agent-search/src/index.rs` with:

```rust
use super::*;

#[test]
fn empty_query_returns_no_results() {
    let index = SearchIndex::new([SearchDocument::new("a", "Alpha")]);
    assert!(index.search(SearchQuery::new("   ")).is_empty());
}
```

- [ ] **Step 2: Verify empty-query test.**

Run:

```bash
cargo nextest run -p agent-search empty_query
```

Expected: PASS.

### Task 3: Verify Full Search Extraction

**Files:**
- Test sidecar: `crates/agent-search/src/*.rs`
- Read: `docs/plans/extract-crates/agent-search.md`

- [ ] **Step 1: Run focused search tests.**

Run:

```bash
cargo nextest run -p agent-search
```

Expected: ranking and filtering tests are deterministic.

- [ ] **Step 2: Scan for gateway/runtime ownership leakage.**

Run:

```bash
rg -n "invoke|call_tool|runtime|reqwest|axum|clap|rmcp" crates/agent-search
```

Expected: no output.
