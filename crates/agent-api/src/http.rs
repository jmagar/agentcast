use crate::{
    GatewayApi, GatewayApiAction, GatewayApiSearchResult, ProtectedMcpJsonRpcRequest,
    ProtectedMcpJsonRpcResponse, ProtectedMcpResponse, ProtectedMcpRouteApi, ResponseStatus,
};
use agent_gateway::ProtectedRouteIndex;
use agent_runtime::McpRuntime;
use axum::{
    Json, Router,
    extract::{OriginalUri, Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

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

impl ApiErrorResponse {
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
