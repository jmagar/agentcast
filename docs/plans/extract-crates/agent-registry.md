---
title: "agent-registry Extraction Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/acp/docs/markdown/0043-agentclientprotocol-com-get-started-registry.md"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md"
  - "docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md"
related: []
last_reviewed: "2026-05-15"
last_modified: "2026-05-15"
modified_on_branch: "gateway-first-skeleton"
modified_at_version: "0.1.0"
modified_at_commit: "d327495"
review_basis: "cross-referenced against gateway-first implementation audit and local docs/references snapshot"
---

# agent-registry Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract registry lookup and metadata normalization for MCP Registry first, with ACP Registry reserved for post-v0.

**Architecture:** `agent-registry` fetches and caches registry metadata. Marketplace turns that metadata into install plans; config writes happen in `agent-config`.

**Tech Stack:** Rust 2024, reqwest, serde, serde_json, url, tracing.

---

## MVP Position

For v0, registry means the official MCP Registry only.

## Current Implementation Audit

As of 2026-05-15 on `gateway-first-skeleton`, `agent-registry` has the v0 MCP Registry foundation extracted. The crate now includes official MCP Registry response DTOs, normalized server/package/remote models with provenance, status, freshness, and publish/update metadata, stable `mcp:{name}` cache keys, an HTTP client for `/v0.1/servers` with search/limit/cursor query support and paginated `list_all_servers`, in-memory cache ownership with fetched-at freshness metadata, deterministic local search, and source-side tests for normalization, pagination, cache replacement, freshness, and search behavior.

Continue with durable registry cache/audit persistence if a file or database-backed cache is promoted into v0. ACP Registry behavior remains post-v0.

## Lab Source Files

- `../lab/crates/lab-apis/src/mcpregistry.rs`
- `../lab/crates/lab-apis/src/acp_registry.rs`
- `../lab/crates/lab/src/api/services/registry_v01.rs`
- `../lab/crates/lab/src/dispatch/marketplace/acp_catalog.rs`
- `../lab/crates/lab/src/dispatch/marketplace/acp_client.rs`
- `../lab/crates/lab/src/dispatch/marketplace/store.rs`
- `../lab/crates/lab/src/dispatch/marketplace/mcp_params.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab registry source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`.
- MCP Registry client/model claims are cross-checked against `docs/references/mcp/docs/markdown/0147-modelcontextprotocol-io-registry-quickstart.md` and `docs/references/mcp/docs/markdown/0064-modelcontextprotocol-io-registry-package-types.md`; the official docs mark the registry as preview.
- ACP Registry is post-v0 and is cross-checked against `docs/references/acp/docs/markdown/0043-agentclientprotocol-com-get-started-registry.md`.

## Live Lab Findings

- `lab-apis/src/mcpregistry.rs` is the canonical live source for MCP Registry DTOs and client behavior.
- `api/services/registry_v01.rs` has useful HTTP query handling for list, versions, and get-by-name.
- `dispatch/marketplace/store.rs` contains the local registry cache, pagination, latest-version filtering, cursor encoding, local metadata, migrations, and restrictive DB permissions.
- ACP registry files are post-v0 evidence and should not block MCP Registry support.

## Extraction Boundary

Extract:

- registry client request/response normalization.
- package/server metadata shapes.
- cache key and refresh patterns.
- error handling for unavailable registry sources.

Leave behind:

- ACP Registry install behavior until post-v0.
- Lab marketplace target assumptions.
- UI catalog cards.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-registry/src/lib.rs` - public exports for MCP registry client, normalized models, cache, search, and errors.
- Create: `crates/agent-registry/src/error.rs` - registry error type and stable kind strings.
- Create: `crates/agent-registry/src/mcp.rs` - official MCP Registry DTOs and normalization.
- Create: `crates/agent-registry/src/client.rs` - HTTP client wrapper with base URL and timeout.
- Create: `crates/agent-registry/src/cache.rs` - cache trait and in-memory cache implementation.
- Create: `crates/agent-registry/src/search.rs` - deterministic fixture-friendly search helpers.
- Add source-side test sidecars for: `crates/agent-registry/src/mcp.rs` - DTO normalization tests.
- Add source-side test sidecars for: `crates/agent-registry/src/search.rs` - local search tests.
- Add source-side test sidecars for: `crates/agent-registry/src/cache.rs` - cache ownership tests.

## Implementation Tasks

### Task 1: Implement MCP Registry Client

**Files:**
- Modify: `crates/agent-registry/src/lib.rs`
- Create: `crates/agent-registry/src/mcp.rs`
- Create: `crates/agent-registry/src/client.rs`
- Create: `crates/agent-registry/src/error.rs`
- Test sidecar: `crates/agent-registry/src/mcp.rs`

- [ ] **Step 1: Inspect Lab MCP registry models.**

Run:

```bash
sed -n '1,260p' ../lab/crates/lab-apis/src/mcpregistry.rs
```

Expected: reusable response shapes and normalization needs are identified.

- [ ] **Step 2: Add a registry fixture constant to the sidecar tests.**

Add this fixture constant inside the source-side test sidecar next to `crates/agent-registry/src/mcp.rs`:

```rust
const MCP_REGISTRY_SERVERS_FIXTURE: &str = r#"{
  "servers": [{
    "name": "io.modelcontextprotocol/filesystem",
    "description": "Filesystem MCP server",
    "version": "0.6.2",
    "packages": [{
      "registry_type": "npm",
      "identifier": "@modelcontextprotocol/server-filesystem",
      "version": "0.6.2",
      "runtime_hint": "npx",
      "transport": "stdio"
    }]
  }]
}"#;
```

- [ ] **Step 3: Write failing registry normalization tests.**

Create a source-side test sidecar next to `crates/agent-registry/src/mcp.rs` with:

```rust
use super::*;

#[test]
fn normalizes_mcp_registry_response() {
    let response: McpRegistryResponse = serde_json::from_str(MCP_REGISTRY_SERVERS_FIXTURE).unwrap();
    let servers = response.normalize().unwrap();

    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "io.modelcontextprotocol/filesystem");
    assert_eq!(servers[0].latest_version.as_deref(), Some("0.6.2"));
    assert_eq!(servers[0].packages[0].identifier, "@modelcontextprotocol/server-filesystem");
}

#[test]
fn normalized_server_has_stable_cache_key() {
    let server = NormalizedMcpServer {
        name: "io.modelcontextprotocol/filesystem".into(),
        description: Some("Filesystem MCP server".into()),
        latest_version: Some("0.6.2".into()),
        packages: vec![],
    };

    assert_eq!(server.cache_key(), "mcp:io.modelcontextprotocol/filesystem");
}
```

Run:

```bash
cargo nextest run -p agent-registry mcp_registry
```

Expected: FAIL because registry DTOs and normalized models do not exist yet.

- [ ] **Step 4: Export registry modules.**

Update `crates/agent-registry/src/lib.rs`:

```rust
mod client;
mod error;
mod mcp;

pub use client::McpRegistryClient;
pub use error::{RegistryError, RegistryResult};
pub use mcp::{
    McpRegistryPackage, McpRegistryResponse, McpRegistryServer, NormalizedMcpPackage,
    NormalizedMcpServer,
};
```

- [ ] **Step 5: Implement error type.**

Create `crates/agent-registry/src/error.rs`:

```rust
use thiserror::Error;

pub type RegistryResult<T> = Result<T, RegistryError>;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("registry request failed: {0}")]
    Request(String),
    #[error("registry data invalid: {0}")]
    InvalidData(String),
}

impl RegistryError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Request(_) => "registry_request",
            Self::InvalidData(_) => "registry_invalid_data",
        }
    }
}
```

- [ ] **Step 6: Implement MCP registry DTOs and normalization.**

Create `crates/agent-registry/src/mcp.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::{RegistryError, RegistryResult};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpRegistryResponse {
    #[serde(default)]
    pub servers: Vec<McpRegistryServer>,
}

impl McpRegistryResponse {
    pub fn normalize(self) -> RegistryResult<Vec<NormalizedMcpServer>> {
        self.servers.into_iter().map(McpRegistryServer::normalize).collect()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpRegistryServer {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub packages: Vec<McpRegistryPackage>,
}

impl McpRegistryServer {
    fn normalize(self) -> RegistryResult<NormalizedMcpServer> {
        if self.name.trim().is_empty() {
            return Err(RegistryError::InvalidData("server name cannot be blank".into()));
        }
        Ok(NormalizedMcpServer {
            name: self.name,
            description: self.description,
            latest_version: self.version,
            packages: self.packages.into_iter().map(McpRegistryPackage::normalize).collect(),
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpRegistryPackage {
    pub registry_type: String,
    pub identifier: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub runtime_hint: Option<String>,
    #[serde(default)]
    pub transport: Option<String>,
}

impl McpRegistryPackage {
    fn normalize(self) -> NormalizedMcpPackage {
        NormalizedMcpPackage {
            registry_type: self.registry_type,
            identifier: self.identifier,
            version: self.version,
            runtime_hint: self.runtime_hint,
            transport: self.transport,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedMcpServer {
    pub name: String,
    pub description: Option<String>,
    pub latest_version: Option<String>,
    pub packages: Vec<NormalizedMcpPackage>,
}

impl NormalizedMcpServer {
    pub fn cache_key(&self) -> String {
        format!("mcp:{}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedMcpPackage {
    pub registry_type: String,
    pub identifier: String,
    pub version: Option<String>,
    pub runtime_hint: Option<String>,
    pub transport: Option<String>,
}
```

- [ ] **Step 7: Implement HTTP client shell.**

Create `crates/agent-registry/src/client.rs`:

```rust
use url::Url;

use crate::{McpRegistryResponse, NormalizedMcpServer, RegistryError, RegistryResult};

#[derive(Clone)]
pub struct McpRegistryClient {
    client: reqwest::Client,
    base_url: Url,
}

impl McpRegistryClient {
    pub fn new(base_url: Url) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    pub async fn list_servers(&self) -> RegistryResult<Vec<NormalizedMcpServer>> {
        let url = self.base_url.join("servers").map_err(|error| {
            RegistryError::Request(format!("invalid registry server URL: {error}"))
        })?;
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|error| RegistryError::Request(error.to_string()))?
            .json::<McpRegistryResponse>()
            .await
            .map_err(|error| RegistryError::InvalidData(error.to_string()))?;
        response.normalize()
    }
}
```

- [ ] **Step 8: Verify MCP registry tests.**

Run:

```bash
cargo nextest run -p agent-registry mcp_registry
```

Expected: PASS.

### Task 2: Add Cache And Search Helpers

**Files:**
- Create: `crates/agent-registry/src/cache.rs`
- Create: `crates/agent-registry/src/search.rs`
- Test sidecar: `crates/agent-registry/src/search.rs`

- [ ] **Step 1: Write failing search tests.**

Create a source-side test sidecar next to `crates/agent-registry/src/search.rs` with:

```rust
use super::*;

fn server(name: &str, description: &str) -> NormalizedMcpServer {
    NormalizedMcpServer {
        name: name.into(),
        description: Some(description.into()),
        latest_version: None,
        packages: vec![],
    }
}

#[test]
fn search_servers_matches_name_and_description() {
    let servers = vec![
        server("io.modelcontextprotocol/filesystem", "Filesystem MCP server"),
        server("io.modelcontextprotocol/git", "Git repository tools"),
    ];

    let results = search_servers(&servers, "file", 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "io.modelcontextprotocol/filesystem");
}

#[test]
fn search_servers_honors_limit() {
    let servers = vec![server("a", "match"), server("b", "match")];
    let results = search_servers(&servers, "match", 1);
    assert_eq!(results.len(), 1);
}
```

- [ ] **Step 2: Export cache and search helpers.**

Update `crates/agent-registry/src/lib.rs`:

```rust
mod cache;
mod search;

pub use cache::{InMemoryRegistryCache, RegistryCache};
pub use search::search_servers;
```

- [ ] **Step 3: Implement deterministic search.**

Create `crates/agent-registry/src/search.rs`:

```rust
use crate::NormalizedMcpServer;

pub fn search_servers(
    servers: &[NormalizedMcpServer],
    query: &str,
    limit: usize,
) -> Vec<NormalizedMcpServer> {
    let needle = query.to_ascii_lowercase();
    servers
        .iter()
        .filter(|server| {
            server.name.to_ascii_lowercase().contains(&needle)
                || server
                    .description
                    .as_deref()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .contains(&needle)
        })
        .take(limit)
        .cloned()
        .collect()
}
```

- [ ] **Step 4: Implement cache trait and memory cache.**

Create `crates/agent-registry/src/cache.rs`:

```rust
use std::collections::BTreeMap;

use crate::NormalizedMcpServer;

pub trait RegistryCache {
    fn put(&mut self, server: NormalizedMcpServer);
    fn get(&self, cache_key: &str) -> Option<&NormalizedMcpServer>;
    fn list(&self) -> Vec<NormalizedMcpServer>;
}

#[derive(Default)]
pub struct InMemoryRegistryCache {
    servers: BTreeMap<String, NormalizedMcpServer>,
}

impl RegistryCache for InMemoryRegistryCache {
    fn put(&mut self, server: NormalizedMcpServer) {
        self.servers.insert(server.cache_key(), server);
    }

    fn get(&self, cache_key: &str) -> Option<&NormalizedMcpServer> {
        self.servers.get(cache_key)
    }

    fn list(&self) -> Vec<NormalizedMcpServer> {
        self.servers.values().cloned().collect()
    }
}
```

- [ ] **Step 5: Verify search tests.**

Run:

```bash
cargo nextest run -p agent-registry search
```

Expected: PASS.

### Task 3: Decide Cache Ownership With `agent-store`

**Files:**
- Modify: `crates/agent-registry/src/cache.rs`
- Modify: `crates/agent-store/src/lib.rs`
- Test sidecar: `crates/agent-registry/src/cache.rs`

- [ ] **Step 1: Read Lab registry store boundaries.**

Run:

```bash
rg -n "RegistryStore|StoreListParams|PagedServers|migrate|restrictive_permissions|cursor|is_latest" ../lab/crates/lab/src/dispatch/marketplace/store.rs
```

Expected: `agent-registry` owns registry semantics; `agent-store` owns SQLite mechanics.

- [ ] **Step 2: Write failing cache tests.**

Create a source-side test sidecar next to `crates/agent-registry/src/cache.rs` with:

```rust
use super::*;

#[test]
fn cache_round_trips_normalized_server_by_cache_key() {
    let mut cache = InMemoryRegistryCache::default();
    let server = NormalizedMcpServer {
        name: "io.modelcontextprotocol/filesystem".into(),
        description: None,
        latest_version: Some("0.6.2".into()),
        packages: vec![],
    };
    let key = server.cache_key();

    cache.put(server);
    assert_eq!(cache.get(&key).unwrap().latest_version.as_deref(), Some("0.6.2"));
}
```

- [ ] **Step 3: Verify cache tests.**

Run:

```bash
cargo nextest run -p agent-registry cache
```

Expected: PASS.

### Task 4: Verify Full Registry Extraction

**Files:**
- Test sidecar: `crates/agent-registry/src/*.rs`
- Read: `docs/plans/extract-crates/agent-registry.md`

- [ ] **Step 1: Run focused registry tests.**

Run:

```bash
cargo nextest run -p agent-registry
```

Expected: PASS.

- [ ] **Step 2: Scan for ACP post-v0 leakage in v0 registry code.**

Run:

```bash
rg -n "acp|Acp|ACP|LAB_|\\.lab|Plex|Sonarr|Radarr|Unraid|Gotify" crates/agent-registry
```

Expected: no output for v0 MCP Registry code.

- [ ] **Step 3: Commit the registry extraction slice.**

Run:

```bash
git add crates/agent-registry docs/plans/extract-crates/agent-registry.md
git commit -m "feat(registry): extract mcp registry client"
```

Expected: commit contains only `agent-registry` implementation, tests, fixtures, and this plan if executing this slice alone.
