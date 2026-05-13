---
title: "agent-api Extraction Implementation Plan"
doc_type: "plan"
status: "draft"
owner: "agentcast"
audience:
  - "implementers"
  - "contributors"
scope: "v0"
source_of_truth: false
upstream_refs:
  - "docs/references/acp/docs/markdown/0022-agentclientprotocol-com-protocol-overview.md"
  - "docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md"
  - "docs/references/jmagar/jmagar-lab.xml"
  - "docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md"
  - "docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md"
  - "docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md"
  - "docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md"
  - "docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md"
  - "docs/references/mcp/docs/markdown/0185-modelcontextprotocol-io-docs-learn-architecture.md"
related: []
last_reviewed: "2026-05-13"
last_modified: "2026-05-13"
modified_on_branch: "main"
modified_at_version: "0.1.0"
modified_at_commit: "unborn"
review_basis: "cross-referenced against local docs/references snapshot"
---

# agent-api Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract Lab's thin Axum API adapter pattern for AgentCast without moving domain behavior into HTTP handlers.

**Architecture:** `agent-api` should assemble routes, request DTOs, response DTOs, OpenAPI metadata, auth extraction, and error envelopes. Runtime behavior stays in `agent-runtime`, `agent-gateway`, `agent-registry`, and `agent-marketplace`.

**Tech Stack:** Rust 2024, Axum, tower-http, serde, utoipa, AgentCast runtime/gateway/registry crates.

---

## MVP Position

`agent-api` is useful after the CLI proves the MCP launcher runtime path. The API should expose the same semantics as the CLI; it must not become the first owner of MCP launcher behavior.

## Lab Source Files

- `../lab/crates/lab/src/api.rs`
- `../lab/crates/lab/src/api/router.rs`
- `../lab/crates/lab/src/api/state.rs`
- `../lab/crates/lab/src/api/error.rs`
- `../lab/crates/lab/src/api/health.rs`
- `../lab/crates/lab/src/api/host_validation.rs`
- `../lab/crates/lab/src/api/openapi.rs`
- `../lab/crates/lab/src/api/services/helpers.rs`
- `../lab/crates/lab/src/api/services/gateway.rs`
- `../lab/crates/lab/src/api/services/marketplace.rs`
- `../lab/crates/lab/src/api/services/registry_v01.rs`
- `../lab/crates/lab/src/api/services/acp.rs`

See also: `docs/reports/lab-extraction-source-map.md`.

## Upstream Reference Checks

- Lab API source claims are cross-checked against `docs/references/jmagar/jmagar-lab.xml`.
- MCP endpoint, lifecycle, transport, tool, resource, and prompt claims are cross-checked against `docs/references/mcp/docs/markdown/0185-modelcontextprotocol-io-docs-learn-architecture.md`, `docs/references/mcp/docs/markdown/0019-modelcontextprotocol-io-specification-2025-11-25-basic-lifecycle.md`, `docs/references/mcp/docs/markdown/0088-modelcontextprotocol-io-specification-2025-11-25-basic-transports.md`, `docs/references/mcp/docs/markdown/0107-modelcontextprotocol-io-specification-2025-11-25-server-tools.md`, `docs/references/mcp/docs/markdown/0045-modelcontextprotocol-io-specification-2025-11-25-server-resources.md`, and `docs/references/mcp/docs/markdown/0175-modelcontextprotocol-io-specification-2025-11-25-server-prompts.md`.
- ACP route references are post-v0 only and are cross-checked against `docs/references/acp/docs/markdown/0022-agentclientprotocol-com-protocol-overview.md` and `docs/references/acp/docs/markdown/0059-agentclientprotocol-com-protocol-schema.md`.

## Live Lab Findings

- `api/router.rs` is rich but too broad; extract route layering, auth placement, host/CORS handling, OpenAPI registration, and route precedence patterns only.
- `api/error.rs` proves error envelopes should map shared domain errors at the surface boundary.
- `api/state.rs` shows how easily app state grows. AgentCast state should contain runtime/gateway handles and avoid Lab node/service fields.
- `api/services/gateway.rs`, `marketplace.rs`, and `registry_v01.rs` are the relevant route families for the MCP launcher MVP.

## Extraction Boundary

Extract:

- route composition patterns.
- shared app state injection.
- action-handler helpers that delegate to shared dispatch/runtime.
- JSON success and error envelope patterns.
- host validation and CORS shape only when appropriate for AgentCast.
- OpenAPI route metadata once endpoints stabilize.

Leave behind:

- Lab homelab service routes.
- browser chat compatibility routes unless ACP is promoted post-v0.
- Lab node/fleet API.
- service credential extraction.

## File Structure

Create or modify these AgentCast files when implementing this plan:

- Modify: `crates/agent-api/src/lib.rs` - public router, state, error, and route exports.
- Create: `crates/agent-api/src/state.rs` - shared app state with injected handles.
- Create: `crates/agent-api/src/error.rs` - JSON error envelope and domain error mapping.
- Create: `crates/agent-api/src/router.rs` - Axum router composition.
- Create: `crates/agent-api/src/routes/mod.rs` - route module exports.
- Create: `crates/agent-api/src/routes/health.rs` - public health route.
- Create: `crates/agent-api/src/routes/actions.rs` - action list/invoke routes.
- Create: `crates/agent-api/src/routes/registry.rs` - registry search route.
- Create: `crates/agent-api/src/routes/install.rs` - install-plan preview/apply routes.
- Add sidecar tests in: `crates/agent-api/src/router.rs` (`#[cfg(test)] mod tests`) - route contract tests against fakes.
- Add sidecar tests in: `crates/agent-api/src/error.rs` (`#[cfg(test)] mod tests`) - error response tests.

## Implementation Tasks

### Task 1: Map API Surfaces To Existing Runtime Semantics

**Files:**
- Read: `docs/MVP.md`
- Read: `docs/API.md`
- Read: `../lab/crates/lab/src/api/services/helpers.rs`
- Modify: `docs/plans/extract-crates/agent-api.md`

- [ ] **Step 1: Verify API endpoint families against MVP.**

Run:

```bash
sed -n '1,220p' docs/API.md
```

Expected: v0 endpoints are MCP servers, launcher actions, launcher invocations, registry, gateway, and config.

- [ ] **Step 2: Inspect Lab helper routing.**

Run:

```bash
sed -n '1,240p' ../lab/crates/lab/src/api/services/helpers.rs
```

Expected: reusable action-handler and error-envelope patterns are identified before code extraction.

### Task 2: Implement Router, State, And Error Envelope

**Files:**
- Modify: `crates/agent-api/src/lib.rs`
- Create: `crates/agent-api/src/router.rs`
- Create: `crates/agent-api/src/state.rs`
- Create: `crates/agent-api/src/error.rs`
- Create: `crates/agent-api/src/routes/mod.rs`
- Create: `crates/agent-api/src/routes/health.rs`
- Test sidecar: `crates/agent-api/src/router.rs` (`#[cfg(test)] mod tests`)
- Test sidecar: `crates/agent-api/src/error.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Write failing health route contract test.**

Add this `#[cfg(test)] mod tests` sidecar to `crates/agent-api/src/router.rs`:

```rust
use super::*;
use axum::body::Body;
use http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn health_route_returns_ok() {
    let app = build_router(ApiState::for_tests());
    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

- [ ] **Step 2: Write failing error envelope test.**

Add this `#[cfg(test)] mod tests` sidecar to `crates/agent-api/src/error.rs`:

```rust
use super::*;

#[test]
fn error_envelope_uses_stable_kind_string() {
    let error = ApiError::bad_request("invalid params");
    let envelope = ErrorEnvelope::from(&error);

    assert_eq!(envelope.error.kind, "bad_request");
    assert_eq!(envelope.error.message, "invalid params");
}
```

Run:

```bash
cargo test -p agent-api
```

Expected: FAIL because the router, state, and error types do not exist yet.

- [ ] **Step 3: Export API modules.**

Update `crates/agent-api/src/lib.rs`:

```rust
mod error;
mod router;
mod routes;
mod state;

pub use error::{ApiError, ApiResult, ErrorBody, ErrorEnvelope};
pub use router::build_router;
pub use state::ApiState;
```

- [ ] **Step 4: Implement API state.**

Create `crates/agent-api/src/state.rs`:

```rust
#[derive(Clone, Default)]
pub struct ApiState {
    pub app_name: String,
}

impl ApiState {
    pub fn for_tests() -> Self {
        Self {
            app_name: "AgentCast test".into(),
        }
    }
}
```

- [ ] **Step 5: Implement error envelope.**

Create `crates/agent-api/src/error.rs`:

```rust
use axum::{Json, http::StatusCode, response::{IntoResponse, Response}};
use serde::Serialize;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Clone)]
pub struct ApiError {
    status: StatusCode,
    kind: &'static str,
    message: String,
}

impl ApiError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            kind: "bad_request",
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ErrorBody {
    pub kind: &'static str,
    pub message: String,
}

impl From<&ApiError> for ErrorEnvelope {
    fn from(error: &ApiError) -> Self {
        Self {
            error: ErrorBody {
                kind: error.kind,
                message: error.message.clone(),
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status;
        let envelope = ErrorEnvelope::from(&self);
        (status, Json(envelope)).into_response()
    }
}
```

- [ ] **Step 6: Implement health route and router.**

Create `crates/agent-api/src/routes/mod.rs`:

```rust
pub mod health;
```

Create `crates/agent-api/src/routes/health.rs`:

```rust
use axum::{Json, extract::State};
use serde::Serialize;

use crate::ApiState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub app: String,
}

pub async fn health(State(state): State<ApiState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        app: state.app_name,
    })
}
```

Create `crates/agent-api/src/router.rs`:

```rust
use axum::{Router, routing::get};

use crate::{ApiState, routes};

pub fn build_router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(routes::health::health))
        .with_state(state)
}
```

- [ ] **Step 7: Verify router and error tests.**

Run:

```bash
cargo test -p agent-api http_contract error_envelope
```

Expected: PASS.

### Task 3: Implement Thin Action, Registry, And Install Routes

**Files:**
- Create: `crates/agent-api/src/routes/actions.rs`
- Create: `crates/agent-api/src/routes/registry.rs`
- Create: `crates/agent-api/src/routes/install.rs`
- Modify: `crates/agent-api/src/routes/mod.rs`
- Modify: `crates/agent-api/src/router.rs`
- Test sidecar: `crates/agent-api/src/router.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Write failing action route test.**

Append this case to the `#[cfg(test)] mod tests` sidecar in `crates/agent-api/src/router.rs`:

```rust
#[tokio::test]
async fn actions_route_returns_empty_fixture_catalog() {
    let app = build_router(ApiState::for_tests());
    let response = app
        .oneshot(Request::builder().uri("/v1/actions").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

- [ ] **Step 2: Add route modules.**

Update `crates/agent-api/src/routes/mod.rs`:

```rust
pub mod actions;
pub mod health;
pub mod install;
pub mod registry;
```

Create `crates/agent-api/src/routes/actions.rs`:

```rust
use axum::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ActionsResponse {
    pub actions: Vec<serde_json::Value>,
}

pub async fn list_actions() -> Json<ActionsResponse> {
    Json(ActionsResponse { actions: vec![] })
}
```

Create `crates/agent-api/src/routes/registry.rs`:

```rust
use axum::{Json, extract::Query};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegistrySearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

#[derive(Debug, Serialize)]
pub struct RegistrySearchResponse {
    pub query: String,
    pub results: Vec<serde_json::Value>,
}

fn default_limit() -> usize {
    20
}

pub async fn search_registry(Query(query): Query<RegistrySearchQuery>) -> Json<RegistrySearchResponse> {
    Json(RegistrySearchResponse {
        query: query.q,
        results: vec![],
    })
}
```

Create `crates/agent-api/src/routes/install.rs`:

```rust
use axum::{Json, extract::Path};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct InstallPreviewResponse {
    pub package: String,
    pub changes: Vec<serde_json::Value>,
}

pub async fn preview_install(Path(package): Path<String>) -> Json<InstallPreviewResponse> {
    Json(InstallPreviewResponse {
        package,
        changes: vec![],
    })
}
```

- [ ] **Step 3: Wire v1 routes.**

Update `crates/agent-api/src/router.rs`:

```rust
pub fn build_router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(routes::health::health))
        .route("/v1/actions", get(routes::actions::list_actions))
        .route("/v1/registry/search", get(routes::registry::search_registry))
        .route("/v1/install/:package/preview", get(routes::install::preview_install))
        .with_state(state)
}
```

- [ ] **Step 4: Verify HTTP route tests.**

Run:

```bash
cargo test -p agent-api http_contract
```

Expected: PASS.

### Task 4: Keep API DTOs Shared With UI Contracts

**Files:**
- Modify: `crates/agent-api/src/routes/`
- Modify: `crates/agent-ui-contracts/src/lib.rs`
- Test sidecar: `crates/agent-api/src/router.rs` (`#[cfg(test)] mod tests`)

- [ ] **Step 1: Check Lab route DTOs before defining AgentCast responses.**

Run:

```bash
rg -n "struct .*Query|struct .*Request|struct .*Response|routes\\(" ../lab/crates/lab/src/api/services/gateway.rs ../lab/crates/lab/src/api/services/marketplace.rs ../lab/crates/lab/src/api/services/registry_v01.rs
```

Expected: shared response shapes are owned by `agent-ui-contracts` when they are consumed by both API and future UI.

- [ ] **Step 2: Move shared DTOs when a route is consumed by UI.**

Expected: route-local structs remain local until `apps/web` or `apps/desktop` consumes them; shared DTOs move to `crates/agent-ui-contracts/src/lib.rs` with serialization tests.

### Task 5: Verify Full API Extraction

**Files:**
- Test sidecar: `crates/agent-api/src/*.rs` (`#[cfg(test)] mod tests`)
- Read: `docs/plans/extract-crates/agent-api.md`

- [ ] **Step 1: Run focused API tests.**

Run:

```bash
cargo test -p agent-api
```

Expected: PASS.

- [ ] **Step 2: Scan for Lab service route leakage.**

Run:

```bash
rg -n "Plex|Sonarr|Radarr|Unraid|Gotify|LAB_|\\.lab|node|fleet|browser_session" crates/agent-api
```

Expected: no output.

- [ ] **Step 3: Commit the API extraction slice.**

Run:

```bash
git add crates/agent-api docs/plans/extract-crates/agent-api.md
git commit -m "feat(api): extract thin http adapter"
```

Expected: commit contains only `agent-api` implementation, tests, and this plan if executing this slice alone.
