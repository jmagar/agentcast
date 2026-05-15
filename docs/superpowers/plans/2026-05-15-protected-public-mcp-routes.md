---
title: "Protected Public MCP Routes Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/plans/extract-crates/gateway-first.md"
  - "docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md"
related:
  - "docs/MVP.md"
  - "docs/TESTING.md"
  - "docs/contracts/crates-and-dependencies.md"
  - "docs/superpowers/plans/2026-05-15-gateway-first-skeleton.md"
last_reviewed: "2026-05-15"
last_modified: "2026-05-15"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "1290747"
review_basis: "superpowers writing-plans pass over gateway-first protected route phases"
---

# Protected Public MCP Routes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add protected public MCP route policy, OAuth protected-resource metadata, and auth-enforcement primitives without implementing upstream OAuth credential lifecycle.

**Architecture:** `agent-gateway` owns protected route config validation, host/path normalization, route indexing, and route-to-upstream policy. `agent-auth` owns protected-resource metadata, scope sets, bearer challenge construction, and generic auth decision outputs. `agent-api` mounts HTTP endpoints later as surface glue over the gateway/auth services and must not duplicate route matching or auth policy.

**Tech Stack:** Rust 2024, `serde`, `thiserror`, `url`, source-side test sidecars, `cargo nextest`, `cargo xtask audit-docs`.

---

## Scope Check

This plan covers phases 5a and 5b from `docs/plans/extract-crates/gateway-first.md`:

- protected route config model.
- host and path normalization.
- duplicate route detection.
- route resolution without Axum.
- protected-resource metadata DTOs.
- bearer challenge and insufficient-scope response primitives.

This plan does not implement:

- Axum route mounting and Streamable HTTP behavior.
- upstream OAuth probe, PKCE state, callback completion, credential storage, refresh, or runtime credential injection.
- generic backend URL proxying beyond a typed route target placeholder for AgentCast-managed upstreams.

## Prerequisites

Complete [2026-05-15-gateway-first-skeleton.md](./2026-05-15-gateway-first-skeleton.md) first. This plan assumes these types already exist:

- `agent_protocol::McpServerId`
- `agent_protocol::McpToolId`
- `agent_protocol::LauncherActionId`
- `agent_runtime::ToolCallRequest`
- `agent_gateway::GatewayError`

## File Structure

- Modify: `crates/agent-auth/src/lib.rs` - public exports for protected-resource and scope modules.
- Create: `crates/agent-auth/src/scope.rs` - scope parsing and matching.
- Create: `crates/agent-auth/src/scope/tests.rs` - source-side scope tests.
- Create: `crates/agent-auth/src/protected_resource.rs` - protected-resource metadata and auth challenge DTOs.
- Create: `crates/agent-auth/src/protected_resource/tests.rs` - source-side metadata/challenge tests.
- Modify: `crates/agent-gateway/src/lib.rs` - public exports for protected routes.
- Create: `crates/agent-gateway/src/protected_routes.rs` - route config, validation, normalization, and index.
- Create: `crates/agent-gateway/src/protected_routes/tests.rs` - source-side route policy tests.
- Modify: `crates/agent-gateway/src/error.rs` - protected route error variants.

## Task 1: Auth Scope Model

**Files:**
- Modify: `crates/agent-auth/src/lib.rs`
- Create: `crates/agent-auth/src/scope.rs`
- Create: `crates/agent-auth/src/scope/tests.rs`

- [ ] **Step 1: Add failing scope tests**

Write `crates/agent-auth/src/scope/tests.rs`:

```rust
use super::*;

#[test]
fn parses_space_separated_scopes_in_sorted_order() {
    let scopes = ScopeSet::parse("mcp:write mcp:read mcp:read").expect("parse scopes");

    assert_eq!(scopes.as_slice(), &["mcp:read", "mcp:write"]);
}

#[test]
fn rejects_empty_scope_segments() {
    let error = ScopeSet::parse("mcp:read  ").expect_err("invalid scopes");

    assert_eq!(error.to_string(), "scope string contains an empty segment");
}

#[test]
fn required_scopes_must_all_be_present() {
    let granted = ScopeSet::parse("mcp:read mcp:write").expect("parse granted");
    let required = ScopeSet::parse("mcp:read").expect("parse required");
    let missing = ScopeSet::parse("mcp:admin").expect("parse missing");

    assert!(granted.contains_all(&required));
    assert!(!granted.contains_all(&missing));
}
```

- [ ] **Step 2: Wire auth exports**

Write `crates/agent-auth/src/lib.rs`:

```rust
pub mod protected_resource;
pub mod scope;

pub use protected_resource::{AuthChallenge, ProtectedResourceMetadata};
pub use scope::{ScopeError, ScopeSet};
```

- [ ] **Step 3: Run tests to verify they fail**

Run:

```bash
cargo nextest run -p agent-auth scope
```

Expected: FAIL because `scope.rs` does not exist yet.

- [ ] **Step 4: Implement scope parsing**

Write `crates/agent-auth/src/scope.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ScopeSet {
    scopes: Vec<String>,
}

impl ScopeSet {
    pub fn parse(raw: &str) -> Result<Self, ScopeError> {
        if raw.is_empty() {
            return Ok(Self { scopes: Vec::new() });
        }

        if raw.split(' ').any(str::is_empty) {
            return Err(ScopeError::EmptySegment);
        }

        let scopes = raw
            .split(' ')
            .map(validate_scope)
            .collect::<Result<BTreeSet<_>, _>>()?
            .into_iter()
            .collect();

        Ok(Self { scopes })
    }

    pub fn from_scopes(scopes: impl IntoIterator<Item = impl Into<String>>) -> Result<Self, ScopeError> {
        let scopes = scopes
            .into_iter()
            .map(|scope| validate_scope(&scope.into()))
            .collect::<Result<BTreeSet<_>, _>>()?
            .into_iter()
            .collect();

        Ok(Self { scopes })
    }

    pub fn as_slice(&self) -> &[String] {
        &self.scopes
    }

    pub fn contains_all(&self, required: &ScopeSet) -> bool {
        required
            .scopes
            .iter()
            .all(|scope| self.scopes.binary_search(scope).is_ok())
    }

    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }

    pub fn as_header_value(&self) -> String {
        self.scopes.join(" ")
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ScopeError {
    #[error("scope string contains an empty segment")]
    EmptySegment,
    #[error("scope `{0}` contains unsupported characters")]
    InvalidCharacters(String),
}

fn validate_scope(scope: &str) -> Result<String, ScopeError> {
    let valid = scope
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, ':' | '.' | '_' | '-'));

    if valid {
        Ok(scope.to_string())
    } else {
        Err(ScopeError::InvalidCharacters(scope.to_string()))
    }
}
```

- [ ] **Step 5: Run scope tests**

Run:

```bash
cargo nextest run -p agent-auth scope
```

Expected: PASS for all scope tests.

- [ ] **Step 6: Commit**

Run:

```bash
git add crates/agent-auth
git commit -m "feat: add auth scope model"
```

Expected: commit includes only `crates/agent-auth`.

## Task 2: Protected Resource Metadata And Challenges

**Files:**
- Modify: `crates/agent-auth/src/lib.rs`
- Create: `crates/agent-auth/src/protected_resource.rs`
- Create: `crates/agent-auth/src/protected_resource/tests.rs`

- [ ] **Step 1: Add failing protected-resource tests**

Write `crates/agent-auth/src/protected_resource/tests.rs`:

```rust
use super::*;
use crate::ScopeSet;

#[test]
fn metadata_serializes_authorization_servers_and_scopes() {
    let metadata = ProtectedResourceMetadata {
        resource: "https://mcp.example.test/syslog".to_string(),
        authorization_servers: vec!["https://auth.example.test".to_string()],
        scopes_supported: ScopeSet::parse("mcp:read mcp:write").expect("parse scopes"),
        bearer_methods_supported: vec!["header".to_string()],
    };

    let value = serde_json::to_value(&metadata).expect("serialize metadata");

    assert_eq!(value["resource"], "https://mcp.example.test/syslog");
    assert_eq!(value["authorization_servers"][0], "https://auth.example.test");
    assert_eq!(value["scopes_supported"][0], "mcp:read");
    assert_eq!(value["bearer_methods_supported"][0], "header");
}

#[test]
fn unauthorized_challenge_points_to_resource_metadata() {
    let challenge = AuthChallenge::unauthorized(
        "https://mcp.example.test/.well-known/oauth-protected-resource/syslog",
    );

    assert_eq!(
        challenge.www_authenticate(),
        "Bearer resource_metadata=\"https://mcp.example.test/.well-known/oauth-protected-resource/syslog\""
    );
}

#[test]
fn insufficient_scope_challenge_includes_required_scopes() {
    let scopes = ScopeSet::parse("mcp:read").expect("parse scopes");
    let challenge = AuthChallenge::insufficient_scope(
        "https://mcp.example.test/.well-known/oauth-protected-resource/syslog",
        scopes,
    );

    assert_eq!(
        challenge.www_authenticate(),
        "Bearer error=\"insufficient_scope\", resource_metadata=\"https://mcp.example.test/.well-known/oauth-protected-resource/syslog\", scope=\"mcp:read\""
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```bash
cargo nextest run -p agent-auth protected_resource
```

Expected: FAIL because `protected_resource.rs` is empty or missing.

- [ ] **Step 3: Implement metadata and challenges**

Write `crates/agent-auth/src/protected_resource.rs`:

```rust
use crate::ScopeSet;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ProtectedResourceMetadata {
    pub resource: String,
    pub authorization_servers: Vec<String>,
    pub scopes_supported: ScopeSet,
    pub bearer_methods_supported: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthChallenge {
    resource_metadata: String,
    error: Option<String>,
    scope: ScopeSet,
}

impl AuthChallenge {
    pub fn unauthorized(resource_metadata: impl Into<String>) -> Self {
        Self {
            resource_metadata: resource_metadata.into(),
            error: None,
            scope: ScopeSet::parse("").expect("empty scope set is valid"),
        }
    }

    pub fn insufficient_scope(resource_metadata: impl Into<String>, scope: ScopeSet) -> Self {
        Self {
            resource_metadata: resource_metadata.into(),
            error: Some("insufficient_scope".to_string()),
            scope,
        }
    }

    pub fn www_authenticate(&self) -> String {
        let mut parts = Vec::new();
        if let Some(error) = &self.error {
            parts.push(format!("error=\"{error}\""));
        }
        parts.push(format!("resource_metadata=\"{}\"", self.resource_metadata));
        if !self.scope.is_empty() {
            parts.push(format!("scope=\"{}\"", self.scope.as_header_value()));
        }
        format!("Bearer {}", parts.join(", "))
    }
}
```

- [ ] **Step 4: Run protected-resource tests**

Run:

```bash
cargo nextest run -p agent-auth protected_resource
```

Expected: PASS for all protected-resource tests.

- [ ] **Step 5: Commit**

Run:

```bash
git add crates/agent-auth
git commit -m "feat: add protected resource metadata"
```

Expected: commit includes only `crates/agent-auth`.

## Task 3: Protected Route Policy

**Files:**
- Modify: `crates/agent-gateway/src/lib.rs`
- Modify: `crates/agent-gateway/src/error.rs`
- Create: `crates/agent-gateway/src/protected_routes.rs`
- Create: `crates/agent-gateway/src/protected_routes/tests.rs`

- [ ] **Step 1: Add failing protected-route tests**

Write `crates/agent-gateway/src/protected_routes/tests.rs`:

```rust
use super::*;
use agent_auth::ScopeSet;
use agent_protocol::McpServerId;

fn route(name: &str, host: &str, path: &str) -> ProtectedRouteConfig {
    ProtectedRouteConfig {
        name: name.to_string(),
        enabled: true,
        public_host: host.to_string(),
        public_path: path.to_string(),
        resource_uri: format!("https://{host}{path}"),
        authorization_servers: vec!["https://auth.example.test".to_string()],
        required_scopes: ScopeSet::parse("mcp:read").expect("parse scopes"),
        target: ProtectedRouteTarget::UpstreamMcp {
            server_id: McpServerId::new("local"),
        },
    }
}

#[test]
fn resolves_by_normalized_host_and_first_path_segment() {
    let index = ProtectedRouteIndex::from_routes(vec![route("syslog", "MCP.Example.Test.", "/syslog")])
        .expect("build index");

    let resolved = index
        .resolve("mcp.example.test:443", "/syslog/tools/list")
        .expect("route resolved");

    assert_eq!(resolved.name, "syslog");
}

#[test]
fn rejects_duplicate_enabled_host_and_path() {
    let error = ProtectedRouteIndex::from_routes(vec![
        route("one", "mcp.example.test", "/syslog"),
        route("two", "MCP.EXAMPLE.TEST.", "/syslog/tools"),
    ])
    .expect_err("duplicate route");

    assert_eq!(
        error.to_string(),
        "duplicate protected route for host `mcp.example.test` and path segment `/syslog`"
    );
}

#[test]
fn rejects_reserved_and_ambiguous_public_paths() {
    for path in ["/", "/.well-known", "/v1", "/syslog//tools", "/syslog/..", "/syslog/%2fsecret"] {
        let error = ProtectedRouteIndex::from_routes(vec![route("bad", "mcp.example.test", path)])
            .expect_err("invalid path");
        assert!(error.to_string().contains("invalid public path"));
    }
}

#[test]
fn exact_metadata_path_resolves_route() {
    let index = ProtectedRouteIndex::from_routes(vec![route("syslog", "mcp.example.test", "/syslog")])
        .expect("build index");

    let resolved = index
        .resolve_metadata("mcp.example.test", "/.well-known/oauth-protected-resource/syslog")
        .expect("metadata route resolved");

    assert_eq!(resolved.name, "syslog");
}
```

- [ ] **Step 2: Export protected route module**

Update `crates/agent-gateway/src/lib.rs`:

```rust
pub mod catalog;
pub mod error;
pub mod protected_routes;
pub mod router;

pub use catalog::{CollisionReport, GatewayCatalog};
pub use error::GatewayError;
pub use protected_routes::{
    ProtectedRouteConfig, ProtectedRouteIndex, ProtectedRouteTarget, ResolvedProtectedRoute,
};
pub use router::{ActionRoute, GatewayRouter};
```

- [ ] **Step 3: Add protected-route error variants**

Update `crates/agent-gateway/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum GatewayError {
    #[error("unknown action `{0}`")]
    UnknownAction(String),
    #[error("invalid public host `{0}`")]
    InvalidPublicHost(String),
    #[error("invalid public path `{0}`")]
    InvalidPublicPath(String),
    #[error("duplicate protected route for host `{host}` and path segment `{path}`")]
    DuplicateProtectedRoute { host: String, path: String },
}
```

- [ ] **Step 4: Run tests to verify they fail**

Run:

```bash
cargo nextest run -p agent-gateway protected_routes
```

Expected: FAIL because `protected_routes.rs` is not implemented yet.

- [ ] **Step 5: Implement protected route policy**

Write `crates/agent-gateway/src/protected_routes.rs`:

```rust
use crate::GatewayError;
use agent_auth::ScopeSet;
use agent_protocol::McpServerId;
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedRouteConfig {
    pub name: String,
    pub enabled: bool,
    pub public_host: String,
    pub public_path: String,
    pub resource_uri: String,
    pub authorization_servers: Vec<String>,
    pub required_scopes: ScopeSet,
    pub target: ProtectedRouteTarget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtectedRouteTarget {
    UpstreamMcp { server_id: McpServerId },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedProtectedRoute {
    pub name: String,
    pub resource_uri: String,
    pub metadata_path: String,
    pub authorization_servers: Vec<String>,
    pub required_scopes: ScopeSet,
    pub target: ProtectedRouteTarget,
}

#[derive(Clone, Debug, Default)]
pub struct ProtectedRouteIndex {
    routes: BTreeMap<(String, String), ResolvedProtectedRoute>,
}

impl ProtectedRouteIndex {
    pub fn from_routes(routes: Vec<ProtectedRouteConfig>) -> Result<Self, GatewayError> {
        let mut index = BTreeMap::new();

        for route in routes.into_iter().filter(|route| route.enabled) {
            let host = normalize_host(&route.public_host)?;
            let segment = first_path_segment(&route.public_path)?;
            let key = (host.clone(), segment.clone());
            if index.contains_key(&key) {
                return Err(GatewayError::DuplicateProtectedRoute {
                    host,
                    path: segment,
                });
            }

            index.insert(
                key,
                ResolvedProtectedRoute {
                    name: route.name,
                    resource_uri: route.resource_uri,
                    metadata_path: format!("/.well-known/oauth-protected-resource{}", segment),
                    authorization_servers: route.authorization_servers,
                    required_scopes: route.required_scopes,
                    target: route.target,
                },
            );
        }

        Ok(Self { routes: index })
    }

    pub fn resolve(&self, host: &str, path: &str) -> Option<&ResolvedProtectedRoute> {
        let host = normalize_host(host).ok()?;
        let segment = first_path_segment(path).ok()?;
        self.routes.get(&(host, segment))
    }

    pub fn resolve_metadata(&self, host: &str, path: &str) -> Option<&ResolvedProtectedRoute> {
        let host = normalize_host(host).ok()?;
        self.routes
            .values()
            .find(|route| self.routes.contains_key(&(host.clone(), route.metadata_path.trim_start_matches("/.well-known/oauth-protected-resource").to_string())) && route.metadata_path == path)
    }
}

fn normalize_host(raw: &str) -> Result<String, GatewayError> {
    let host = raw
        .split(',')
        .next()
        .unwrap_or_default()
        .trim()
        .trim_end_matches('.')
        .to_ascii_lowercase();
    let host = host.split(':').next().unwrap_or_default().to_string();

    if host.is_empty() || host.contains('/') || host.contains('\\') {
        return Err(GatewayError::InvalidPublicHost(raw.to_string()));
    }

    Ok(host)
}

fn first_path_segment(raw: &str) -> Result<String, GatewayError> {
    if !raw.starts_with('/') || raw == "/" {
        return Err(GatewayError::InvalidPublicPath(raw.to_string()));
    }
    let lower = raw.to_ascii_lowercase();
    if lower.starts_with("/.well-known")
        || lower.starts_with("/v1")
        || lower.contains("//")
        || lower.contains("/.")
        || lower.contains("%2f")
        || lower.contains("%5c")
        || lower.contains("%2e")
        || raw.contains('?')
        || raw.contains('#')
    {
        return Err(GatewayError::InvalidPublicPath(raw.to_string()));
    }

    let segment = raw
        .trim_start_matches('/')
        .split('/')
        .next()
        .unwrap_or_default();

    if segment.is_empty() {
        return Err(GatewayError::InvalidPublicPath(raw.to_string()));
    }

    Ok(format!("/{segment}"))
}
```

- [ ] **Step 6: Run protected-route tests**

Run:

```bash
cargo nextest run -p agent-gateway protected_routes
```

Expected: PASS for all protected-route tests.

- [ ] **Step 7: Commit**

Run:

```bash
git add crates/agent-gateway
git commit -m "feat: add protected MCP route policy"
```

Expected: commit includes only `crates/agent-gateway`.

## Task 4: Metadata Projection From Protected Routes

**Files:**
- Modify: `crates/agent-gateway/src/protected_routes.rs`
- Modify: `crates/agent-gateway/src/protected_routes/tests.rs`

- [ ] **Step 1: Add failing metadata projection test**

Append to `crates/agent-gateway/src/protected_routes/tests.rs`:

```rust
#[test]
fn projects_protected_resource_metadata() {
    let index = ProtectedRouteIndex::from_routes(vec![route("syslog", "mcp.example.test", "/syslog")])
        .expect("build index");
    let resolved = index
        .resolve_metadata("mcp.example.test", "/.well-known/oauth-protected-resource/syslog")
        .expect("metadata route resolved");

    let metadata = resolved.protected_resource_metadata();

    assert_eq!(metadata.resource, "https://mcp.example.test/syslog");
    assert_eq!(metadata.authorization_servers[0], "https://auth.example.test");
    assert_eq!(metadata.scopes_supported.as_slice(), &["mcp:read"]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```bash
cargo nextest run -p agent-gateway projects_protected_resource_metadata
```

Expected: FAIL because `protected_resource_metadata` is not implemented yet.

- [ ] **Step 3: Implement metadata projection**

Add this impl block to `crates/agent-gateway/src/protected_routes.rs`:

```rust
impl ResolvedProtectedRoute {
    pub fn protected_resource_metadata(&self) -> agent_auth::ProtectedResourceMetadata {
        agent_auth::ProtectedResourceMetadata {
            resource: self.resource_uri.clone(),
            authorization_servers: self.authorization_servers.clone(),
            scopes_supported: self.required_scopes.clone(),
            bearer_methods_supported: vec!["header".to_string()],
        }
    }
}
```

- [ ] **Step 4: Run route tests**

Run:

```bash
cargo nextest run -p agent-gateway protected_routes
```

Expected: PASS for all protected-route tests.

- [ ] **Step 5: Commit**

Run:

```bash
git add crates/agent-gateway
git commit -m "feat: expose protected resource metadata"
```

Expected: commit includes only `crates/agent-gateway`.

## Task 5: Verification

**Files:**
- Read: `docs/plans/extract-crates/gateway-first.md`
- Read: `docs/contracts/crates-and-dependencies.md`
- Read: `docs/TESTING.md`

- [ ] **Step 1: Run focused tests**

Run:

```bash
cargo nextest run -p agent-auth
cargo nextest run -p agent-gateway protected_routes
```

Expected: all focused tests pass.

- [ ] **Step 2: Run dependency-boundary grep**

Run:

```bash
rg -n "rmcp|agent_client_protocol|axum|tower|clap|rusqlite|sqlx" crates/agent-gateway crates/agent-auth
```

Expected: no matches for direct SDK/storage imports in `agent-gateway` or `agent-auth`.

- [ ] **Step 3: Run docs audit**

Run:

```bash
cargo xtask audit-docs
```

Expected: `audit-docs: checked` with no errors.

- [ ] **Step 4: Run pre-push-sized verification**

Run:

```bash
cargo xtask ci
```

Expected: format check, workspace check, clippy, and nextest pass.

- [ ] **Step 5: Commit this plan if it has not already been committed**

Run:

```bash
git add docs/superpowers/plans/2026-05-15-protected-public-mcp-routes.md
git commit -m "docs: add protected MCP routes implementation plan"
```

Expected: commit includes only this plan document unless implementation commits intentionally include it earlier.

## Self-Review

Spec coverage:

- Route config, host/path normalization, duplicate detection, and route resolution are covered by Task 3.
- Protected-resource metadata and bearer challenges are covered by Tasks 2 and 4.
- Scope parsing and scope matching are covered by Task 1.
- API mounting, Streamable HTTP, and upstream OAuth lifecycle are intentionally excluded and require separate plans.

Placeholder scan:

- This plan avoids placeholder markers and unnamed validation steps.
- Every code-changing step includes concrete code for the target file.

Type consistency:

- `ScopeSet` is defined in Task 1 and reused in Tasks 2-4.
- `ProtectedResourceMetadata` is defined in Task 2 and projected from `ResolvedProtectedRoute` in Task 4.
- `ProtectedRouteConfig`, `ProtectedRouteIndex`, and `ResolvedProtectedRoute` are defined in Task 3 and reused consistently in Task 4.
