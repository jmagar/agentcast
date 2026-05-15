use crate::{GatewayApi, GatewayApiAction, GatewayApiSearchResult};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
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
