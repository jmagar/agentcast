use crate::{
    GatewayApi, GatewayApiAction, GatewayApiSearchResult, OAuthStatusResponse,
    ProtectedMcpJsonRpcRequest, ProtectedMcpJsonRpcResponse, ProtectedMcpResponse,
    ProtectedMcpRouteApi, ResponseStatus,
};
use agent_auth::{OAuthCallback, OAuthCredential, OAuthProviderMetadata, ScopeSet};
use agent_gateway::{BeginAuthorization, GatewayOAuthService, ProtectedRouteIndex};
use agent_runtime::McpRuntime;
use agent_store::InMemoryOAuthStore;
use axum::{
    Json, Router,
    extract::{OriginalUri, Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(test)]
mod tests;

pub fn gateway_router(api: GatewayApi) -> Router {
    Router::new()
        .route("/v1/gateway/actions", get(list_actions))
        .route("/v1/gateway/search", get(search_actions))
        .route("/v1/gateway/actions/{action_id}/call", post(call_action))
        .route("/v1/gateway/resources/read", post(read_resource))
        .with_state(Arc::new(api))
}

pub fn oauth_router() -> Router {
    Router::new()
        .route("/v1/oauth/probe", post(oauth_probe))
        .route("/v1/oauth/authorize", post(oauth_authorize))
        .route("/v1/oauth/callback", post(oauth_callback))
        .route("/v1/oauth/status", get(oauth_status))
        .route("/v1/oauth/credentials", delete(oauth_clear))
        .with_state(Arc::new(OAuthHttpState {
            service: Mutex::new(GatewayOAuthService::new(InMemoryOAuthStore::default())),
        }))
}

pub fn protected_mcp_router(routes: ProtectedRouteIndex, runtime: Arc<McpRuntime>) -> Router {
    let state = ProtectedMcpHttpState {
        api: ProtectedMcpRouteApi::new(routes),
        runtime,
    };

    Router::new()
        .route(
            "/.well-known/oauth-protected-resource/{*route}",
            get(protected_metadata),
        )
        .route("/{*route}", post(protected_json_rpc))
        .with_state(Arc::new(state))
}

async fn list_actions(State(api): State<Arc<GatewayApi>>) -> Json<Vec<GatewayApiAction>> {
    Json(api.list_actions())
}

async fn search_actions(
    State(api): State<Arc<GatewayApi>>,
    Query(query): Query<SearchActionsQuery>,
) -> Json<Vec<GatewayApiSearchResult>> {
    Json(api.search_actions(&query.q, query.limit.unwrap_or(10)))
}

async fn call_action(
    State(api): State<Arc<GatewayApi>>,
    Path(action_id): Path<String>,
    Json(request): Json<CallActionRequest>,
) -> Result<Json<Value>, ApiErrorResponse> {
    api.call_action(&action_id, request.arguments)
        .await
        .map(Json)
        .map_err(|error| ApiErrorResponse::bad_gateway(error.to_string()))
}

async fn read_resource(
    State(api): State<Arc<GatewayApi>>,
    Json(request): Json<ReadResourceRequest>,
) -> Result<Json<Value>, ApiErrorResponse> {
    api.read_resource(&request.server_id, &request.uri)
        .await
        .map(Json)
        .map_err(|error| ApiErrorResponse::bad_gateway(error.to_string()))
}

async fn oauth_probe(
    State(state): State<Arc<OAuthHttpState>>,
    Json(request): Json<OAuthProbeRequest>,
) -> Result<Json<OAuthProbeResponse>, ApiErrorResponse> {
    let service = state.service.lock().await;
    let status = service
        .probe_metadata(&request.issuer_url, &request.metadata)
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    Ok(Json(OAuthProbeResponse {
        status: status.into(),
    }))
}

async fn oauth_authorize(
    State(state): State<Arc<OAuthHttpState>>,
    Json(request): Json<OAuthAuthorizeRequest>,
) -> Result<Json<OAuthAuthorizeResponse>, ApiErrorResponse> {
    let challenge_scopes = parse_optional_scope(request.challenge_scopes.as_deref())?;
    let protected_resource_scopes =
        parse_optional_scope(request.protected_resource_scopes.as_deref())?;
    let begin = BeginAuthorization {
        issuer_url: request.issuer_url,
        subject: request.subject,
        upstream_id: request.upstream_id,
        client_id: request.client_id,
        redirect_uri: request.redirect_uri,
        resource_uri: request.resource_uri,
        state: request.state,
        code_challenge: request.code_challenge,
        expires_at_unix: request.expires_at_unix,
        challenge_scopes,
        protected_resource_scopes,
    };

    let mut service = state.service.lock().await;
    let result = service
        .begin_authorization(begin, &request.metadata)
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    Ok(Json(OAuthAuthorizeResponse {
        authorization_url: result.authorization_url,
        selected_scopes: result
            .selected_scope
            .map(|scopes| scopes.as_slice().to_vec())
            .unwrap_or_default(),
    }))
}

async fn oauth_callback(
    State(state): State<Arc<OAuthHttpState>>,
    Json(request): Json<OAuthCallbackRequest>,
) -> Result<StatusCode, ApiErrorResponse> {
    let mut service = state.service.lock().await;
    service
        .complete_callback(
            OAuthCallback {
                state: request.state,
                subject: request.subject,
                code: request.code,
            },
            request.credential,
            request.now_unix,
        )
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn oauth_status(
    State(state): State<Arc<OAuthHttpState>>,
    Query(query): Query<OAuthStatusQuery>,
) -> Result<Json<OAuthStatusResponse>, ApiErrorResponse> {
    let service = state.service.lock().await;
    let status = service
        .status(
            &query.subject,
            &query.upstream_id,
            query.now_unix.unwrap_or(0),
        )
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    Ok(Json(OAuthStatusResponse::from_status(
        &query.subject,
        &query.upstream_id,
        status,
    )))
}

async fn oauth_clear(
    State(state): State<Arc<OAuthHttpState>>,
    Query(query): Query<OAuthClearQuery>,
) -> Result<StatusCode, ApiErrorResponse> {
    let mut service = state.service.lock().await;
    service
        .clear(&query.subject, &query.upstream_id)
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

fn parse_optional_scope(raw: Option<&str>) -> Result<Option<ScopeSet>, ApiErrorResponse> {
    raw.map(ScopeSet::parse)
        .transpose()
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))
}

async fn protected_metadata(
    State(state): State<Arc<ProtectedMcpHttpState>>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
) -> Response {
    match state
        .api
        .handle(protected_request(&headers, uri.path(), None))
    {
        ProtectedMcpResponse::Metadata { status, metadata } => {
            (status_code(status), Json(metadata)).into_response()
        }
        response => protected_rejection(response).into_response(),
    }
}

async fn protected_json_rpc(
    State(state): State<Arc<ProtectedMcpHttpState>>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    let response = state
        .api
        .handle_json_rpc(
            &state.runtime,
            ProtectedMcpJsonRpcRequest {
                host: host(&headers),
                path: uri.path().to_string(),
                public_origin: public_origin(&headers),
                authorization: authorization(&headers),
                body,
            },
        )
        .await;

    match response {
        ProtectedMcpJsonRpcResponse::JsonRpc(body) => Json(body).into_response(),
        ProtectedMcpJsonRpcResponse::Rejected(response) => {
            protected_rejection(response).into_response()
        }
    }
}

fn protected_request(
    headers: &HeaderMap,
    path: &str,
    authorization_override: Option<String>,
) -> crate::ProtectedMcpRequest {
    crate::ProtectedMcpRequest {
        host: host(headers),
        path: path.to_string(),
        public_origin: public_origin(headers),
        authorization: authorization_override.or_else(|| authorization(headers)),
    }
}

fn protected_rejection(response: ProtectedMcpResponse) -> Response {
    match response {
        ProtectedMcpResponse::Challenge {
            status,
            www_authenticate,
        } => (
            status_code(status),
            [(header::WWW_AUTHENTICATE, www_authenticate)],
        )
            .into_response(),
        ProtectedMcpResponse::NotFound { status } => status_code(status).into_response(),
        ProtectedMcpResponse::DispatchAllowed { status, .. } => status_code(status).into_response(),
        ProtectedMcpResponse::Metadata { status, metadata } => {
            (status_code(status), Json(metadata)).into_response()
        }
    }
}

fn host(headers: &HeaderMap) -> String {
    headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}

fn public_origin(headers: &HeaderMap) -> String {
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("http");
    format!("{proto}://{}", host(headers))
}

fn authorization(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

fn status_code(status: ResponseStatus) -> StatusCode {
    match status {
        ResponseStatus::Ok => StatusCode::OK,
        ResponseStatus::Accepted => StatusCode::ACCEPTED,
        ResponseStatus::Unauthorized => StatusCode::UNAUTHORIZED,
        ResponseStatus::Forbidden => StatusCode::FORBIDDEN,
        ResponseStatus::NotFound => StatusCode::NOT_FOUND,
    }
}

#[derive(Debug, Deserialize)]
struct SearchActionsQuery {
    q: String,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct CallActionRequest {
    #[serde(default)]
    pub arguments: Value,
}

#[derive(Debug, Deserialize)]
pub struct ReadResourceRequest {
    pub server_id: String,
    pub uri: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthProbeRequest {
    pub issuer_url: String,
    pub metadata: OAuthProviderMetadata,
}

#[derive(Debug, Serialize)]
pub struct OAuthProbeResponse {
    pub status: crate::OAuthApiStatus,
}

#[derive(Debug, Deserialize)]
pub struct OAuthAuthorizeRequest {
    pub issuer_url: String,
    pub metadata: OAuthProviderMetadata,
    pub subject: String,
    pub upstream_id: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub resource_uri: String,
    pub state: String,
    pub code_challenge: String,
    pub expires_at_unix: u64,
    pub challenge_scopes: Option<String>,
    pub protected_resource_scopes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OAuthAuthorizeResponse {
    pub authorization_url: String,
    pub selected_scopes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackRequest {
    pub state: String,
    pub subject: String,
    pub code: String,
    pub credential: OAuthCredential,
    pub now_unix: u64,
}

#[derive(Debug, Deserialize)]
struct OAuthStatusQuery {
    subject: String,
    upstream_id: String,
    now_unix: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OAuthClearQuery {
    subject: String,
    upstream_id: String,
}

#[derive(Debug, Serialize)]
struct ApiErrorBody {
    error: String,
}

#[derive(Debug)]
struct ApiErrorResponse {
    status: StatusCode,
    message: String,
}

#[derive(Debug)]
struct ProtectedMcpHttpState {
    api: ProtectedMcpRouteApi,
    runtime: Arc<McpRuntime>,
}

#[derive(Debug)]
struct OAuthHttpState {
    service: Mutex<GatewayOAuthService<InMemoryOAuthStore>>,
}

impl ApiErrorResponse {
    fn bad_request(message: String) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message,
        }
    }

    fn bad_gateway(message: String) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            message,
        }
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}
