---
title: "agent-auth Extraction Implementation Plan"
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
  - "docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "b941533"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-auth Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract only generic authentication primitives needed for AgentCast local and HTTP surfaces.

**Architecture:** `agent-auth` should provide bearer/session identity extraction and shared authorization primitives. Product policy remains in API/server code; Lab-specific Google OAuth and homelab admin policy do not move by default.

**Tech Stack:** Rust 2024, Axum extractors/middleware, rusqlite when durable sessions are needed, serde, sha2/hex.

---

## MVP Position

For v0, auth can stay minimal because the MCP launcher is local-first. HTTP API auth should be thin and explicit once the API is exposed beyond loopback.

The gateway-first v0 scope promotes generic MCP protected-resource metadata and upstream OAuth primitives into v0. This does not promote Lab's Google OAuth product flow, browser session UX, admin policy, or homelab service auth; it only means `agent-auth` must provide generic protected-resource metadata, bearer/scope helpers, PKCE/state/callback validation, token domain types, and credential encryption primitives or an auth-facing repository trait for `agent-store`.

## Lab Source Files

- `../lab/crates/lab-auth/src/auth_context.rs`
- `../lab/crates/lab-auth/src/routes.rs`
- `../lab/crates/lab-auth/src/config.rs`
- `../lab/crates/lab-auth/src/error.rs`
- `../lab/crates/lab-auth/src/middleware.rs`
- `../lab/crates/lab-auth/src/session.rs`
- `../lab/crates/lab-auth/src/sqlite.rs`
- `../lab/crates/lab-auth/src/state.rs`
- `../lab/crates/lab-auth/src/token.rs`
- `../lab/crates/lab/src/api/auth_helpers.rs`
- `../lab/crates/lab/src/api/browser_session.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab auth source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`; the packed snapshot contains `crates/lab-auth/src/routes.rs` rather than a standalone `authorize.rs` file entry.
- MCP HTTP authorization claims are cross-checked against `docs/references/mcp/docs/markdown/0184-modelcontextprotocol-io-specification-2025-11-25-basic-authorization.md`.
- ACP authentication and initialization metadata claims are cross-checked against `docs/references/acp/docs/markdown/0011-agentclientprotocol-com-protocol-initialization.md` and `docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md`.

## Live Lab Findings

- `auth_context.rs` is the cleanest source for principal propagation and `WWW-Authenticate` metadata helpers.
- `middleware.rs`, `config.rs`, and `error.rs` contain reusable auth-mode, bearer/session, and stable error-kind patterns.
- `session.rs`, `state.rs`, `sqlite.rs`, `token.rs`, and `types.rs` are useful for post-v0 browser/API auth, but Lab Google OAuth policy should not move into v0.
- `api/router.rs` shows how auth layers interact with public health routes and protected MCP routes; keep that as API/server evidence, not auth product policy.

## Extraction Boundary

Extract:

- `AuthContext` shape and principal propagation.
- bearer token middleware patterns.
- protected-resource metadata DTOs and `WWW-Authenticate` helpers for gateway-served MCP routes.
- scope parsing and insufficient-scope error helpers.
- PKCE/state/callback validation primitives for upstream OAuth flows.
- token/credential domain types and redaction helpers.
- session ID generation and hashing patterns when an HTTP session store is introduced.
- authorization helper patterns that return typed errors.

Leave behind:

- Lab Google OAuth flow.
- Lab OAuth env var names and hard-coded upstream exceptions.
- Lab admin/service policy.
- browser session routes unless AgentCast ships a browser UI.
- node/fleet enrollment auth.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-auth/src/lib.rs` - public exports for context, bearer, metadata, and errors.
- Create: `crates/agent-auth/src/context.rs` - authenticated principal and scope model.
- Create: `crates/agent-auth/src/error.rs` - auth error type and stable kind strings.
- Create: `crates/agent-auth/src/bearer.rs` - bearer-token validation and Axum extraction helpers.
- Create: `crates/agent-auth/src/metadata.rs` - auth metadata DTOs without Google/Lab policy.
- Add source-side test sidecars for: `crates/agent-auth/src/context.rs` - identity propagation tests.
- Add source-side test sidecars for: `crates/agent-auth/src/bearer.rs` - bearer validator tests.
- Add source-side test sidecars for: `crates/agent-auth/src/metadata.rs` - OAuth/auth metadata serialization tests.

## Implementation Tasks

### Task 1: Define Minimal Identity Contract

**Files:**
- Modify: `crates/agent-auth/src/lib.rs`
- Create: `crates/agent-auth/src/context.rs`
- Create: `crates/agent-auth/src/error.rs`
- Test sidecar: `crates/agent-auth/src/context.rs`

- [ ] **Step 1: Write failing tests for principal propagation.**

Create a source-side test sidecar next to `crates/agent-auth/src/context.rs` with:

```rust
use super::*;

#[test]
fn auth_context_keeps_principal_and_scopes() {
    let context = AuthContext::new("local-user").with_scope("mcp:read").with_scope("mcp:call");

    assert_eq!(context.principal(), "local-user");
    assert!(context.has_scope("mcp:read"));
    assert!(context.has_scope("mcp:call"));
    assert!(!context.has_scope("admin"));
}

#[test]
fn anonymous_context_has_stable_principal() {
    let context = AuthContext::anonymous();
    assert_eq!(context.principal(), "anonymous");
    assert!(context.scopes().is_empty());
}
```

Run:

```bash
cargo nextest run -p agent-auth context
```

Expected: FAIL because `AuthContext` does not exist yet.

- [ ] **Step 2: Export context and error modules.**

Update `crates/agent-auth/src/lib.rs`:

```rust
mod context;
mod error;

pub use context::AuthContext;
pub use error::{AuthError, AuthResult};
```

- [ ] **Step 3: Implement auth error type.**

Create `crates/agent-auth/src/error.rs`:

```rust
use thiserror::Error;

pub type AuthResult<T> = Result<T, AuthError>;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("missing credentials")]
    MissingCredentials,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("forbidden: {0}")]
    Forbidden(String),
}

impl AuthError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::MissingCredentials => "missing_credentials",
            Self::InvalidCredentials => "invalid_credentials",
            Self::Forbidden(_) => "forbidden",
        }
    }
}
```

- [ ] **Step 4: Implement generic auth context.**

Create `crates/agent-auth/src/context.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthContext {
    principal: String,
    scopes: Vec<String>,
}

impl AuthContext {
    pub fn new(principal: impl Into<String>) -> Self {
        Self {
            principal: principal.into(),
            scopes: Vec::new(),
        }
    }

    pub fn anonymous() -> Self {
        Self::new("anonymous")
    }

    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.push(scope.into());
        self
    }

    pub fn principal(&self) -> &str {
        &self.principal
    }

    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|existing| existing == scope)
    }
}
```

- [ ] **Step 5: Verify context tests.**

Run:

```bash
cargo nextest run -p agent-auth context
```

Expected: PASS.


### Task 2: Add HTTP Bearer Middleware When API Needs It

**Files:**
- Create: `crates/agent-auth/src/bearer.rs`
- Test sidecar: `crates/agent-auth/src/bearer.rs`

- [ ] **Step 1: Inspect Lab middleware.**

Run:

```bash
sed -n '1,260p' ../lab/crates/lab-auth/src/middleware.rs
```

Expected: middleware behavior is understood before implementation.

- [ ] **Step 2: Write failing bearer validator tests.**

Create a source-side test sidecar next to `crates/agent-auth/src/bearer.rs` with:

```rust
use super::*;

#[test]
fn bearer_validator_accepts_exact_token() {
    let validator = BearerValidator::new("secret-token");
    let context = validator.validate_header("Bearer secret-token").unwrap();

    assert_eq!(context.principal(), "bearer");
    assert!(context.has_scope("local:api"));
}

#[test]
fn bearer_validator_rejects_missing_prefix() {
    let validator = BearerValidator::new("secret-token");
    let err = validator.validate_header("secret-token").unwrap_err();

    assert!(matches!(err, AuthError::InvalidCredentials));
    assert_eq!(err.kind(), "invalid_credentials");
}
```

- [ ] **Step 3: Export bearer validator.**

Update `crates/agent-auth/src/lib.rs`:

```rust
mod bearer;
pub use bearer::BearerValidator;
```

- [ ] **Step 4: Implement bearer validator.**

Create `crates/agent-auth/src/bearer.rs`:

```rust
use crate::{AuthContext, AuthError, AuthResult};

#[derive(Debug, Clone)]
pub struct BearerValidator {
    token: String,
}

impl BearerValidator {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }

    pub fn validate_header(&self, header: &str) -> AuthResult<AuthContext> {
        let Some(value) = header.strip_prefix("Bearer ") else {
            return Err(AuthError::InvalidCredentials);
        };
        if value != self.token {
            return Err(AuthError::InvalidCredentials);
        }
        Ok(AuthContext::new("bearer").with_scope("local:api"))
    }
}
```

- [ ] **Step 5: Verify bearer tests.**

Run:

```bash
cargo nextest run -p agent-auth bearer
```

Expected: PASS.

### Task 3: Preserve OAuth Metadata Without Google Policy

**Files:**
- Create: `crates/agent-auth/src/metadata.rs`
- Test sidecar: `crates/agent-auth/src/metadata.rs`

- [ ] **Step 1: Read generic metadata and auth-context sources.**

Run:

```bash
sed -n '1,120p' ../lab/crates/lab-auth/src/auth_context.rs
sed -n '1,120p' ../lab/crates/lab-auth/src/types.rs
```

Expected: AgentCast can emit standard auth metadata DTOs without importing Lab Google OAuth allowlist policy.

- [ ] **Step 2: Write failing metadata serialization tests.**

Create a source-side test sidecar next to `crates/agent-auth/src/metadata.rs` with:

```rust
use super::*;

#[test]
fn metadata_serializes_standard_bearer_scheme() {
    let metadata = AuthMetadata {
        schemes: vec![AuthScheme::Bearer {
            realm: "AgentCast".into(),
        }],
    };

    let value = serde_json::to_value(metadata).unwrap();
    assert_eq!(value["schemes"][0]["type"], "bearer");
    assert_eq!(value["schemes"][0]["realm"], "AgentCast");
}
```

- [ ] **Step 3: Export metadata types.**

Update `crates/agent-auth/src/lib.rs`:

```rust
mod metadata;
pub use metadata::{AuthMetadata, AuthScheme};
```

- [ ] **Step 4: Implement metadata DTOs.**

Create `crates/agent-auth/src/metadata.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthMetadata {
    pub schemes: Vec<AuthScheme>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthScheme {
    Bearer { realm: String },
}
```

- [ ] **Step 5: Verify metadata tests.**

Run:

```bash
cargo nextest run -p agent-auth metadata
```

Expected: PASS.

### Task 4: Verify Full Auth Extraction

**Files:**
- Test sidecar: `crates/agent-auth/src/*.rs`
- Read: `docs/plans/extract-crates/agent-auth.md`

- [ ] **Step 1: Run focused auth tests.**

Run:

```bash
cargo nextest run -p agent-auth
```

Expected: PASS.

- [ ] **Step 2: Scan for Lab OAuth policy leakage.**

Run:

```bash
rg -n "Google|OAuth|LAB_|\\.lab|admin_allowlist|Plex|Sonarr|Radarr|Unraid|Gotify" crates/agent-auth
```

Expected: no output for v0 auth primitives.

- [ ] **Step 3: Commit the auth extraction slice.**

Run:

```bash
git add crates/agent-auth docs/plans/extract-crates/agent-auth.md
git commit -m "feat(auth): extract generic auth primitives"
```

Expected: commit contains only `agent-auth` implementation, tests, and this plan if executing this slice alone.
