use crate::{
    GatewayApi, GatewayApiAction, GatewayApiPrompt, GatewayApiResource, GatewayApiSearchResult,
    GatewayApiServer, GatewayApiStatus, OAuthStatusResponse, ProtectedMcpJsonRpcRequest,
    ProtectedMcpJsonRpcResponse, ProtectedMcpRequest, ProtectedMcpResponse, ProtectedMcpRouteApi,
    ResponseStatus,
};
use agent_auth::{
    BearerTokenVerifier, OAuthCallback, OAuthClientRegistration, OAuthCredential,
    OAuthProviderMetadata, OAuthRefreshRequest, OAuthRefreshResult, ScopeSet,
};
use agent_config::AgentConfig;
use agent_gateway::{
    BeginAuthorization, GatewayOAuthService, HttpOAuthTokenClient, OAuthAuthorizationCodeEndpoint,
    OAuthDynamicClientRegistrationEndpoint, OAuthDynamicClientRegistrationRequest,
    OAuthMetadataDiscoveryEndpoint, OAuthRefreshProvider, OAuthRefreshProviderError,
    OAuthTokenRefreshEndpoint, ProtectedRouteCollection, ProtectedRouteConfig, ProtectedRouteIndex,
    ProtectedRouteStatus, ProtectedRouteTarget,
};
use agent_marketplace::{ApplyInstallPlanResult, InstallPlan};
use agent_registry::{NormalizedMcpServer, RegistryCache, search_servers};
use agent_runtime::McpRuntime;
use agent_store::{InMemoryOAuthStore, OAuthStore};
use axum::{
    Json, Router,
    extract::{OriginalUri, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{delete, get, post},
};
use futures::stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::Infallible,
    sync::{Arc, Mutex as StdMutex},
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

#[cfg(test)]
mod tests;

pub fn gateway_router(api: GatewayApi) -> Router {
    Router::new()
        .route("/v1/gateway/status", get(gateway_status))
        .route("/v1/gateway/servers", get(list_servers))
        .route("/v1/gateway/actions", get(list_actions))
        .route("/v1/gateway/search", get(search_actions))
        .route("/v1/gateway/actions/{action_id}/call", post(call_action))
        .route("/v1/gateway/resources", get(list_resources))
        .route("/v1/gateway/resources/read", post(read_resource))
        .route("/v1/gateway/prompts", get(list_prompts))
        .route("/v1/gateway/prompts/get", post(get_prompt))
        .with_state(Arc::new(api))
}

pub fn oauth_router() -> Router {
    oauth_router_with_store(InMemoryOAuthStore::default())
}

pub fn registry_router(servers: Vec<NormalizedMcpServer>) -> Router {
    Router::new()
        .route("/v1/registry/search", get(registry_search))
        .with_state(Arc::new(RegistryHttpState { servers }))
}

pub fn registry_router_from_cache(cache: &impl RegistryCache) -> Router {
    registry_router(cache.list())
}

pub fn marketplace_router() -> Router {
    Router::new()
        .route("/v1/marketplace/mcp/plan", post(marketplace_plan_mcp))
        .route("/v1/marketplace/mcp/apply", post(marketplace_apply_mcp))
}

pub fn oauth_router_with_store<S>(store: S) -> Router
where
    S: OAuthStore + Send + 'static,
{
    Router::new()
        .route("/v1/oauth/probe", post(oauth_probe))
        .route("/v1/oauth/authorize", post(oauth_authorize))
        .route("/v1/oauth/callback", post(oauth_callback))
        .route("/v1/oauth/register", post(oauth_register))
        .route("/v1/oauth/refresh", post(oauth_refresh))
        .route("/v1/oauth/status", get(oauth_status))
        .route("/v1/oauth/credentials", delete(oauth_clear))
        .with_state(Arc::new(OAuthHttpState {
            service: shared_oauth_service(store),
            refresh_locks: SharedRefreshLocks::default(),
        }))
}

pub fn protected_mcp_router(routes: ProtectedRouteIndex, runtime: Arc<McpRuntime>) -> Router {
    protected_mcp_router_with_oauth_service(routes, runtime, None)
}

pub fn protected_mcp_router_with_verifier(
    routes: ProtectedRouteIndex,
    runtime: Arc<McpRuntime>,
    verifier: Arc<dyn BearerTokenVerifier>,
) -> Router {
    protected_mcp_router_with_oauth_service_and_verifier(routes, runtime, None, verifier)
}

pub fn protected_route_admin_router(routes: ProtectedRouteCollection) -> Router {
    Router::new()
        .route("/v1/protected-routes", get(protected_routes_list))
        .route("/v1/protected-routes", post(protected_routes_upsert))
        .route(
            "/v1/protected-routes/{name}",
            get(protected_routes_get).delete(protected_routes_delete),
        )
        .route(
            "/v1/protected-routes/{name}/status",
            get(protected_routes_status),
        )
        .route("/v1/protected-routes/test", post(protected_routes_test))
        .with_state(Arc::new(ProtectedRouteAdminState {
            routes: StdMutex::new(routes),
        }))
}

pub fn protected_mcp_router_with_oauth_store<S>(
    routes: ProtectedRouteIndex,
    runtime: Arc<McpRuntime>,
    store: S,
) -> Router
where
    S: OAuthStore + Send + 'static,
{
    protected_mcp_router_with_oauth_service(routes, runtime, Some(shared_oauth_service(store)))
}

fn protected_mcp_router_with_oauth_service(
    routes: ProtectedRouteIndex,
    runtime: Arc<McpRuntime>,
    oauth: Option<SharedOAuthService>,
) -> Router {
    protected_mcp_router_with_oauth_service_and_verifier(
        routes,
        runtime,
        oauth,
        Arc::new(agent_auth::StaticBearerTokenVerifier::default()),
    )
}

fn protected_mcp_router_with_oauth_service_and_verifier(
    routes: ProtectedRouteIndex,
    runtime: Arc<McpRuntime>,
    oauth: Option<SharedOAuthService>,
    verifier: Arc<dyn BearerTokenVerifier>,
) -> Router {
    let state = ProtectedMcpHttpState {
        api: ProtectedMcpRouteApi::new_with_verifier(routes, verifier),
        runtime,
        oauth,
        sessions: Arc::new(Mutex::new(McpSessions::default())),
    };

    Router::new()
        .route(
            "/.well-known/oauth-protected-resource/{*route}",
            get(protected_metadata),
        )
        .route(
            "/{*route}",
            post(protected_json_rpc)
                .get(protected_sse_get)
                .delete(protected_delete_session),
        )
        .with_state(Arc::new(state))
}

async fn list_actions(State(api): State<Arc<GatewayApi>>) -> Json<Vec<GatewayApiAction>> {
    Json(api.list_actions())
}

async fn gateway_status(State(api): State<Arc<GatewayApi>>) -> Json<GatewayApiStatus> {
    Json(api.status())
}

async fn list_servers(State(api): State<Arc<GatewayApi>>) -> Json<Vec<GatewayApiServer>> {
    Json(api.list_servers())
}

async fn search_actions(
    State(api): State<Arc<GatewayApi>>,
    Query(query): Query<SearchActionsQuery>,
) -> Json<Vec<GatewayApiSearchResult>> {
    Json(api.search_actions(&query.q, bounded_limit(query.limit, DEFAULT_SEARCH_LIMIT)))
}

async fn list_resources(
    State(api): State<Arc<GatewayApi>>,
    Query(query): Query<ListByServerQuery>,
) -> Json<Vec<GatewayApiResource>> {
    Json(api.list_resources(query.server_id.as_deref()))
}

async fn registry_search(
    State(state): State<Arc<RegistryHttpState>>,
    Query(query): Query<RegistrySearchQuery>,
) -> Json<Vec<NormalizedMcpServer>> {
    Json(search_servers(
        &state.servers,
        &query.q,
        bounded_limit(query.limit, DEFAULT_REGISTRY_LIMIT),
    ))
}

async fn marketplace_plan_mcp(
    Json(request): Json<MarketplaceMcpPlanRequest>,
) -> Result<Json<InstallPlan>, ApiErrorResponse> {
    agent_marketplace::plan_mcp_server_install(&request.server)
        .map(Json)
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))
}

async fn marketplace_apply_mcp(
    Json(request): Json<MarketplaceMcpApplyRequest>,
) -> Result<Json<MarketplaceMcpApplyResponse>, ApiErrorResponse> {
    let plan = agent_marketplace::plan_mcp_server_install(&request.server)
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    let env_resolution =
        agent_marketplace::resolve_install_env(&request.server, &request.env_values)
            .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    agent_marketplace::install_env_merge(&env_resolution)
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    let mut config = request.config.unwrap_or_default();
    let result = agent_marketplace::apply_install_plan_to_config(&mut config, &plan)
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    Ok(Json(MarketplaceMcpApplyResponse {
        plan,
        result,
        config,
        env_values: env_resolution.values,
    }))
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

async fn list_prompts(
    State(api): State<Arc<GatewayApi>>,
    Query(query): Query<ListByServerQuery>,
) -> Json<Vec<GatewayApiPrompt>> {
    Json(api.list_prompts(query.server_id.as_deref()))
}

async fn get_prompt(
    State(api): State<Arc<GatewayApi>>,
    Json(request): Json<GetPromptRequest>,
) -> Result<Json<Value>, ApiErrorResponse> {
    api.get_prompt(&request.server_id, &request.name, request.arguments)
        .await
        .map(Json)
        .map_err(|error| ApiErrorResponse::bad_gateway(error.to_string()))
}

async fn oauth_probe(
    State(state): State<Arc<OAuthHttpState>>,
    Json(request): Json<OAuthProbeRequest>,
) -> Result<Json<OAuthProbeResponse>, ApiErrorResponse> {
    let metadata = resolve_oauth_metadata(&request.issuer_url, request.metadata).await?;
    let service = state.service.lock().await;
    let status = service
        .probe_metadata(&request.issuer_url, &metadata)
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    Ok(Json(OAuthProbeResponse {
        status: status.into(),
    }))
}

async fn oauth_authorize(
    State(state): State<Arc<OAuthHttpState>>,
    Json(request): Json<OAuthAuthorizeRequest>,
) -> Result<Json<OAuthAuthorizeResponse>, ApiErrorResponse> {
    let metadata = resolve_oauth_metadata(&request.issuer_url, request.metadata).await?;
    let challenge_scopes = parse_optional_scope(request.challenge_scopes.as_deref())?;
    let protected_resource_scopes =
        parse_optional_scope(request.protected_resource_scopes.as_deref())?;
    let begin = BeginAuthorization {
        issuer_url: request.issuer_url.clone(),
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
        .begin_authorization(begin, &metadata)
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    Ok(Json(OAuthAuthorizeResponse {
        authorization_url: result.authorization_url,
        selected_scopes: result
            .selected_scope
            .map(|scopes| scopes.as_slice().to_vec())
            .unwrap_or_default(),
    }))
}

async fn resolve_oauth_metadata(
    issuer_url: &str,
    metadata: Option<OAuthProviderMetadata>,
) -> Result<OAuthProviderMetadata, ApiErrorResponse> {
    if let Some(metadata) = metadata {
        return Ok(metadata);
    }
    let endpoint = OAuthMetadataDiscoveryEndpoint::new(issuer_url.to_string())
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    HttpOAuthTokenClient::new()
        .discover_metadata(&endpoint)
        .await
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))
}

async fn oauth_callback(
    State(state): State<Arc<OAuthHttpState>>,
    Json(request): Json<OAuthCallbackRequest>,
) -> Result<StatusCode, ApiErrorResponse> {
    let callback = OAuthCallback {
        state: request.state,
        subject: request.subject,
        code: request.code,
    };

    if let Some(credential) = request.credential {
        let mut service = state.service.lock().await;
        service
            .complete_callback(callback, credential, request.now_unix)
            .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
        return Ok(StatusCode::NO_CONTENT);
    }

    let endpoint = request
        .token_endpoint
        .zip(request.client_id)
        .zip(request.redirect_uri)
        .zip(request.code_verifier)
        .map(
            |(((token_endpoint, client_id), redirect_uri), code_verifier)| {
                OAuthAuthorizationCodeEndpoint::new(
                    token_endpoint,
                    client_id,
                    request.client_secret,
                    redirect_uri,
                    code_verifier,
                )
            },
        )
        .transpose()
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?
        .ok_or_else(|| {
            ApiErrorResponse::bad_request(
                "token_endpoint, client_id, redirect_uri, and code_verifier are required for live OAuth callback exchange".to_string(),
            )
        })?;

    let pending = {
        let mut service = state.service.lock().await;
        service
            .begin_callback_exchange(&callback, request.now_unix)
            .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?
    };

    let client = HttpOAuthTokenClient::new();
    let result = client
        .exchange_authorization_code(&endpoint, &callback, request.now_unix)
        .await
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;

    let mut service = state.service.lock().await;
    service
        .finish_callback_exchange(pending, result)
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn oauth_register(
    State(state): State<Arc<OAuthHttpState>>,
    Json(request): Json<OAuthRegisterHttpRequest>,
) -> Result<Json<OAuthClientRegistrationResponse>, ApiErrorResponse> {
    let registration = if let Some(result) = request.result {
        result
    } else {
        let endpoint = OAuthDynamicClientRegistrationEndpoint::new(request.registration_endpoint)
            .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
        HttpOAuthTokenClient::new()
            .register_client(
                &endpoint,
                OAuthDynamicClientRegistrationRequest {
                    subject: request.subject.clone(),
                    upstream_id: request.upstream_id.clone(),
                    redirect_uris: request.redirect_uris,
                    client_name: request.client_name,
                },
            )
            .await
            .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?
    };

    let mut service = state.service.lock().await;
    service
        .put_client_registration(registration.clone())
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    Ok(Json(OAuthClientRegistrationResponse::from_registration(
        registration,
    )))
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

async fn oauth_refresh(
    State(state): State<Arc<OAuthHttpState>>,
    Json(request): Json<OAuthRefreshHttpRequest>,
) -> Result<Json<OAuthStatusResponse>, ApiErrorResponse> {
    let Some(_refresh_guard) = state
        .refresh_locks
        .try_acquire(request.subject.clone(), request.upstream_id.clone())
    else {
        let service = state.service.lock().await;
        let status = service
            .status(&request.subject, &request.upstream_id, request.now_unix)
            .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
        return Ok(Json(OAuthStatusResponse::from_status(
            &request.subject,
            &request.upstream_id,
            status,
        )));
    };

    if let Some(result) = request.result {
        let mut service = state.service.lock().await;
        let mut provider = RequestRefreshProvider {
            result: Some(result),
        };
        let status = service
            .refresh(
                &request.subject,
                &request.upstream_id,
                request.now_unix,
                &mut provider,
            )
            .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
        return Ok(Json(OAuthStatusResponse::from_status(
            &request.subject,
            &request.upstream_id,
            status,
        )));
    }

    let endpoint = request
        .token_endpoint
        .zip(request.client_id)
        .map(|(token_endpoint, client_id)| {
            OAuthTokenRefreshEndpoint::new(token_endpoint, client_id, request.client_secret)
        })
        .transpose()
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?
        .ok_or_else(|| {
            ApiErrorResponse::bad_request(
                "token_endpoint and client_id are required for live OAuth refresh".to_string(),
            )
        })?;

    let refresh_request = {
        let mut service = state.service.lock().await;
        match service
            .begin_refresh(&request.subject, &request.upstream_id, request.now_unix)
            .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?
        {
            Some(refresh_request) => refresh_request,
            None => {
                return Ok(Json(OAuthStatusResponse::from_status(
                    &request.subject,
                    &request.upstream_id,
                    agent_auth::OAuthStatus::RefreshFailed,
                )));
            }
        }
    };

    let client = HttpOAuthTokenClient::new();
    let result = match client.refresh(&endpoint, refresh_request).await {
        Ok(result) => result,
        Err(error) => {
            let mut service = state.service.lock().await;
            service
                .mark_refresh_failed(&request.subject, &request.upstream_id)
                .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
            return Err(ApiErrorResponse::bad_request(error.to_string()));
        }
    };

    let mut service = state.service.lock().await;
    let status = service
        .finish_refresh_success(&request.subject, &request.upstream_id, result)
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    Ok(Json(OAuthStatusResponse::from_status(
        &request.subject,
        &request.upstream_id,
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

async fn protected_routes_list(
    State(state): State<Arc<ProtectedRouteAdminState>>,
) -> Result<Json<Vec<ProtectedRouteConfig>>, ApiErrorResponse> {
    let routes = state
        .routes
        .lock()
        .map_err(|_| ApiErrorResponse::bad_gateway("protected route state lock poisoned".into()))?;
    Ok(Json(routes.list().to_vec()))
}

async fn protected_routes_get(
    State(state): State<Arc<ProtectedRouteAdminState>>,
    Path(name): Path<String>,
) -> Result<Json<ProtectedRouteConfig>, ApiErrorResponse> {
    let routes = state
        .routes
        .lock()
        .map_err(|_| ApiErrorResponse::bad_gateway("protected route state lock poisoned".into()))?;
    routes
        .get(&name)
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiErrorResponse::not_found(format!("protected route `{name}` not found")))
}

async fn protected_routes_upsert(
    State(state): State<Arc<ProtectedRouteAdminState>>,
    Json(route): Json<ProtectedRouteConfig>,
) -> Result<Json<ProtectedRouteConfig>, ApiErrorResponse> {
    let mut routes = state
        .routes
        .lock()
        .map_err(|_| ApiErrorResponse::bad_gateway("protected route state lock poisoned".into()))?;
    routes
        .upsert(route.clone())
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    Ok(Json(route))
}

async fn protected_routes_delete(
    State(state): State<Arc<ProtectedRouteAdminState>>,
    Path(name): Path<String>,
) -> Result<Json<ProtectedRouteConfig>, ApiErrorResponse> {
    let mut routes = state
        .routes
        .lock()
        .map_err(|_| ApiErrorResponse::bad_gateway("protected route state lock poisoned".into()))?;
    routes
        .remove(&name)
        .map(Json)
        .map_err(|error| ApiErrorResponse::not_found(error.to_string()))
}

async fn protected_routes_status(
    State(state): State<Arc<ProtectedRouteAdminState>>,
    Path(name): Path<String>,
) -> Result<Json<ProtectedRouteStatus>, ApiErrorResponse> {
    let routes = state
        .routes
        .lock()
        .map_err(|_| ApiErrorResponse::bad_gateway("protected route state lock poisoned".into()))?;
    routes
        .status(&name)
        .map(Json)
        .map_err(|error| ApiErrorResponse::not_found(error.to_string()))
}

async fn protected_routes_test(
    State(state): State<Arc<ProtectedRouteAdminState>>,
    Json(request): Json<ProtectedRouteTestRequest>,
) -> Result<Json<ProtectedRouteTestResponse>, ApiErrorResponse> {
    let routes = state
        .routes
        .lock()
        .map_err(|_| ApiErrorResponse::bad_gateway("protected route state lock poisoned".into()))?;
    let resolved = routes
        .test(&request.host, &request.path)
        .map_err(|error| ApiErrorResponse::bad_request(error.to_string()))?;
    Ok(Json(ProtectedRouteTestResponse {
        matched: resolved.is_some(),
        route_name: resolved.map(|route| route.name),
    }))
}

struct RequestRefreshProvider {
    result: Option<OAuthRefreshResult>,
}

impl OAuthRefreshProvider for RequestRefreshProvider {
    fn refresh(
        &mut self,
        _request: OAuthRefreshRequest,
    ) -> Result<OAuthRefreshResult, OAuthRefreshProviderError> {
        self.result.take().ok_or(OAuthRefreshProviderError::Failed)
    }
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
        .handle(protected_request(&headers, uri.path(), &state, None))
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
    if let Err(rejection) = validate_mcp_transport_headers(&state, &headers, uri.path()) {
        return mcp_transport_rejection(rejection);
    }
    if let Err(rejection) = validate_known_session(&state, &headers).await {
        return mcp_transport_rejection(rejection);
    }
    let notification = is_json_rpc_notification(&body);
    let initialize = is_json_rpc_initialize_request(&body);
    let request = ProtectedMcpJsonRpcRequest {
        host: host(&headers),
        path: uri.path().to_string(),
        public_origin: request_public_origin(&state, &headers, uri.path()),
        authorization: authorization(&headers),
        body,
    };
    let upstream_access_token = match upstream_access_token_for_request(&state, &request).await {
        Ok(token) => token,
        Err(error) => return error.into_response(),
    };
    let response = state
        .api
        .handle_json_rpc_with_upstream_credential(&state.runtime, request, upstream_access_token)
        .await;

    match response {
        ProtectedMcpJsonRpcResponse::JsonRpc(_body) if notification => {
            StatusCode::ACCEPTED.into_response()
        }
        ProtectedMcpJsonRpcResponse::JsonRpc(body) => {
            let mut response = Json(body.clone()).into_response();
            if initialize && body.get("result").is_some() {
                set_new_session_header(&state, &mut response).await;
            }
            response
        }
        ProtectedMcpJsonRpcResponse::Rejected(response) => {
            protected_rejection(response).into_response()
        }
    }
}

async fn protected_sse_get(
    State(state): State<Arc<ProtectedMcpHttpState>>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
) -> Response {
    if let Err(rejection) = validate_mcp_transport_headers(&state, &headers, uri.path()) {
        return mcp_transport_rejection(rejection);
    }
    if let Err(rejection) = validate_known_session(&state, &headers).await {
        return mcp_transport_rejection(rejection);
    }

    let authorization = state
        .api
        .handle(protected_request(&headers, uri.path(), &state, None));
    if !matches!(authorization, ProtectedMcpResponse::DispatchAllowed { .. }) {
        return protected_rejection(authorization);
    }

    let session_id = match session_id(&headers) {
        Ok(Some(session_id)) => session_id,
        Ok(None) => new_session_id(&state).await,
        Err(rejection) => return mcp_transport_rejection(rejection),
    };
    let last_event_id = headers
        .get("last-event-id")
        .and_then(|value| value.to_str().ok())
        .filter(|value| valid_session_id(value))
        .unwrap_or("0");
    let event_id = next_sse_event_id(last_event_id);
    let event = Event::default()
        .id(event_id)
        .event("endpoint")
        .data(uri.path());

    let mut response = Sse::new(stream::once(async move { Ok::<Event, Infallible>(event) }))
        .keep_alive(KeepAlive::default())
        .into_response();
    if let Ok(value) = HeaderValue::from_str(&session_id) {
        response.headers_mut().insert(MCP_SESSION_ID, value);
    }
    response
}

async fn protected_delete_session(
    State(state): State<Arc<ProtectedMcpHttpState>>,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
) -> Response {
    let authorization = state
        .api
        .handle(protected_request(&headers, uri.path(), &state, None));
    if !matches!(authorization, ProtectedMcpResponse::DispatchAllowed { .. }) {
        return protected_rejection(authorization);
    }

    let session_id = match session_id(&headers) {
        Ok(Some(session_id)) => session_id,
        Ok(None) => return mcp_transport_rejection(McpTransportRejection::InvalidSessionId),
        Err(rejection) => return mcp_transport_rejection(rejection),
    };

    let mut sessions = state.sessions.lock().await;
    if sessions.remove(&session_id) {
        StatusCode::NO_CONTENT.into_response()
    } else {
        mcp_transport_rejection(McpTransportRejection::UnknownSessionId)
    }
}

async fn upstream_access_token_for_request(
    state: &ProtectedMcpHttpState,
    request: &ProtectedMcpJsonRpcRequest,
) -> Result<Option<String>, ApiErrorResponse> {
    let Some(oauth) = &state.oauth else {
        return Ok(None);
    };

    let authorization = state.api.handle(ProtectedMcpRequest {
        host: request.host.clone(),
        path: request.path.clone(),
        public_origin: request.public_origin.clone(),
        authorization: request.authorization.clone(),
    });

    let ProtectedMcpResponse::DispatchAllowed {
        subject,
        target: ProtectedRouteTarget::UpstreamMcp { server_id },
        ..
    } = authorization
    else {
        return Ok(None);
    };

    let service = oauth.lock().await;
    service
        .credential_for_runtime(&subject, server_id.as_str())
        .map(|credential| credential.map(|credential| credential.access_token))
        .map_err(|error| ApiErrorResponse::bad_gateway(error.to_string()))
}

fn validate_mcp_transport_headers(
    state: &ProtectedMcpHttpState,
    headers: &HeaderMap,
    path: &str,
) -> Result<(), McpTransportRejection> {
    if !valid_origin(state, headers, path) {
        return Err(McpTransportRejection::ForbiddenOrigin);
    }

    if !accepts_mcp_response(headers) {
        return Err(McpTransportRejection::NotAcceptable);
    }

    if !valid_protocol_version(headers) {
        return Err(McpTransportRejection::UnsupportedProtocolVersion);
    }

    session_id(headers)?;

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum McpTransportRejection {
    ForbiddenOrigin,
    NotAcceptable,
    UnsupportedProtocolVersion,
    InvalidSessionId,
    UnknownSessionId,
}

fn mcp_transport_rejection(rejection: McpTransportRejection) -> Response {
    let (status, error) = match rejection {
        McpTransportRejection::ForbiddenOrigin => {
            (StatusCode::FORBIDDEN, "invalid protected MCP origin")
        }
        McpTransportRejection::NotAcceptable => (
            StatusCode::NOT_ACCEPTABLE,
            "protected MCP route requires Accept: application/json or text/event-stream",
        ),
        McpTransportRejection::UnsupportedProtocolVersion => {
            (StatusCode::BAD_REQUEST, "unsupported MCP protocol version")
        }
        McpTransportRejection::InvalidSessionId => {
            (StatusCode::BAD_REQUEST, "invalid MCP session id")
        }
        McpTransportRejection::UnknownSessionId => {
            (StatusCode::NOT_FOUND, "unknown MCP session id")
        }
    };
    (
        status,
        Json(ApiErrorEnvelope {
            error: ApiErrorBody {
                code: mcp_rejection_code(status).to_string(),
                message: error.to_string(),
            },
        }),
    )
        .into_response()
}

fn mcp_rejection_code(status: StatusCode) -> &'static str {
    match status {
        StatusCode::FORBIDDEN => "forbidden",
        StatusCode::NOT_ACCEPTABLE => "not_acceptable",
        StatusCode::NOT_FOUND => "not_found",
        _ => "bad_request",
    }
}

fn accepts_mcp_response(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|raw| {
            raw.split(',').any(|part| {
                let media = part
                    .trim()
                    .split_once(';')
                    .map(|(media, _)| media)
                    .unwrap_or(part)
                    .trim();
                matches!(
                    media,
                    "*/*" | "application/*" | "application/json" | "text/event-stream"
                )
            })
        })
}

fn valid_protocol_version(headers: &HeaderMap) -> bool {
    headers
        .get("mcp-protocol-version")
        .and_then(|value| value.to_str().ok())
        .is_none_or(|version| version == "2025-11-25")
}

fn valid_origin(state: &ProtectedMcpHttpState, headers: &HeaderMap, path: &str) -> bool {
    headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .is_none_or(|origin| {
            state
                .api
                .resource_origin(&host(headers), path)
                .is_some_and(|allowed_origin| origin == allowed_origin)
        })
}

fn session_id(headers: &HeaderMap) -> Result<Option<String>, McpTransportRejection> {
    headers
        .get(MCP_SESSION_ID)
        .and_then(|value| value.to_str().ok())
        .map(|session_id| {
            if valid_session_id(session_id) {
                Ok(session_id.to_string())
            } else {
                Err(McpTransportRejection::InvalidSessionId)
            }
        })
        .transpose()
}

fn valid_session_id(session_id: &str) -> bool {
    !session_id.is_empty() && session_id.bytes().all(|byte| (0x21..=0x7e).contains(&byte))
}

async fn validate_known_session(
    state: &ProtectedMcpHttpState,
    headers: &HeaderMap,
) -> Result<(), McpTransportRejection> {
    let Some(session_id) = session_id(headers)? else {
        return Ok(());
    };
    let mut sessions = state.sessions.lock().await;
    sessions.prune_expired();
    if sessions.contains(&session_id) {
        Ok(())
    } else {
        Err(McpTransportRejection::UnknownSessionId)
    }
}

fn is_json_rpc_notification(body: &Value) -> bool {
    body.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
        && body.get("id").is_none()
        && body.get("method").and_then(Value::as_str).is_some()
}

fn is_json_rpc_initialize_request(body: &Value) -> bool {
    body.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
        && body.get("id").is_some()
        && body.get("method").and_then(Value::as_str) == Some("initialize")
}

async fn set_new_session_header(state: &ProtectedMcpHttpState, response: &mut Response) {
    let session_id = new_session_id(state).await;
    if let Ok(value) = HeaderValue::from_str(&session_id) {
        response.headers_mut().insert(MCP_SESSION_ID, value);
    }
}

async fn new_session_id(state: &ProtectedMcpHttpState) -> String {
    let session_id = uuid::Uuid::new_v4().to_string();
    state.sessions.lock().await.insert(session_id.clone());
    session_id
}

fn bounded_limit(requested: Option<usize>, default: usize) -> usize {
    requested.unwrap_or(default).clamp(1, MAX_COLLECTION_LIMIT)
}

fn next_sse_event_id(last_event_id: &str) -> String {
    last_event_id
        .parse::<u64>()
        .ok()
        .and_then(|value| value.checked_add(1))
        .unwrap_or(1)
        .to_string()
}

fn protected_request(
    headers: &HeaderMap,
    path: &str,
    state: &ProtectedMcpHttpState,
    authorization_override: Option<String>,
) -> crate::ProtectedMcpRequest {
    crate::ProtectedMcpRequest {
        host: host(headers),
        path: path.to_string(),
        public_origin: request_public_origin(state, headers, path),
        authorization: authorization_override.or_else(|| authorization(headers)),
    }
}

fn request_public_origin(state: &ProtectedMcpHttpState, headers: &HeaderMap, path: &str) -> String {
    state
        .api
        .resource_origin(&host(headers), path)
        .unwrap_or_else(|| public_origin(headers))
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
struct ListByServerQuery {
    server_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RegistrySearchQuery {
    q: String,
    limit: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarketplaceMcpPlanRequest {
    pub server: NormalizedMcpServer,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarketplaceMcpApplyRequest {
    pub server: NormalizedMcpServer,
    #[serde(default)]
    pub config: Option<AgentConfig>,
    #[serde(default)]
    pub env_values: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MarketplaceMcpApplyResponse {
    pub plan: InstallPlan,
    pub result: ApplyInstallPlanResult,
    pub config: AgentConfig,
    pub env_values: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct ProtectedRouteTestRequest {
    host: String,
    path: String,
}

#[derive(Debug, Serialize)]
struct ProtectedRouteTestResponse {
    matched: bool,
    route_name: Option<String>,
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
pub struct GetPromptRequest {
    pub server_id: String,
    pub name: String,
    #[serde(default)]
    pub arguments: Option<serde_json::Map<String, Value>>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthProbeRequest {
    pub issuer_url: String,
    pub metadata: Option<OAuthProviderMetadata>,
}

#[derive(Debug, Serialize)]
pub struct OAuthProbeResponse {
    pub status: crate::OAuthApiStatus,
}

#[derive(Debug, Deserialize)]
pub struct OAuthAuthorizeRequest {
    pub issuer_url: String,
    pub metadata: Option<OAuthProviderMetadata>,
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
    pub credential: Option<OAuthCredential>,
    pub now_unix: u64,
    pub token_endpoint: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
    pub code_verifier: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthRegisterHttpRequest {
    pub subject: String,
    pub upstream_id: String,
    #[serde(default)]
    pub registration_endpoint: String,
    #[serde(default)]
    pub redirect_uris: Vec<String>,
    pub client_name: Option<String>,
    pub result: Option<OAuthClientRegistration>,
}

#[derive(Debug, Serialize)]
pub struct OAuthClientRegistrationResponse {
    pub subject: String,
    pub upstream_id: String,
    pub client_id: String,
    pub has_client_secret: bool,
    pub client_id_issued_at_unix: Option<u64>,
    pub client_secret_expires_at_unix: Option<u64>,
}

impl OAuthClientRegistrationResponse {
    fn from_registration(registration: OAuthClientRegistration) -> Self {
        Self {
            subject: registration.subject,
            upstream_id: registration.upstream_id,
            client_id: registration.client_id,
            has_client_secret: registration.client_secret.is_some(),
            client_id_issued_at_unix: registration.client_id_issued_at_unix,
            client_secret_expires_at_unix: registration.client_secret_expires_at_unix,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OAuthRefreshHttpRequest {
    pub subject: String,
    pub upstream_id: String,
    pub now_unix: u64,
    pub token_endpoint: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub result: Option<OAuthRefreshResult>,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ApiErrorEnvelope {
    pub error: ApiErrorBody,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiErrorResponse {
    status: StatusCode,
    code: &'static str,
    message: String,
}

struct ProtectedMcpHttpState {
    api: ProtectedMcpRouteApi,
    runtime: Arc<McpRuntime>,
    oauth: Option<SharedOAuthService>,
    sessions: SharedMcpSessions,
}

struct ProtectedRouteAdminState {
    routes: StdMutex<ProtectedRouteCollection>,
}

struct OAuthHttpState {
    service: SharedOAuthService,
    refresh_locks: SharedRefreshLocks,
}

#[derive(Clone)]
struct RegistryHttpState {
    servers: Vec<NormalizedMcpServer>,
}

type SharedOAuthService = Arc<Mutex<GatewayOAuthService<Box<dyn OAuthStore + Send>>>>;
type SharedMcpSessions = Arc<Mutex<McpSessions>>;

const MCP_SESSION_ID: &str = "mcp-session-id";
const DEFAULT_SEARCH_LIMIT: usize = 10;
const DEFAULT_REGISTRY_LIMIT: usize = 20;
const MAX_COLLECTION_LIMIT: usize = 100;
const MCP_SESSION_TTL: Duration = Duration::from_secs(60 * 30);
const MAX_MCP_SESSIONS: usize = 1024;

#[derive(Debug, Default)]
struct McpSessions {
    sessions: BTreeMap<String, Instant>,
}

impl McpSessions {
    fn insert(&mut self, session_id: String) {
        self.prune_expired();
        if self.sessions.len() >= MAX_MCP_SESSIONS
            && let Some(oldest) = self
                .sessions
                .iter()
                .min_by_key(|(_, expires_at)| *expires_at)
                .map(|(session_id, _)| session_id.clone())
        {
            self.sessions.remove(&oldest);
        }
        self.sessions
            .insert(session_id, Instant::now() + MCP_SESSION_TTL);
    }

    fn contains(&self, session_id: &str) -> bool {
        self.sessions
            .get(session_id)
            .is_some_and(|expires_at| *expires_at > Instant::now())
    }

    fn remove(&mut self, session_id: &str) -> bool {
        self.sessions.remove(session_id).is_some()
    }

    fn prune_expired(&mut self) {
        let now = Instant::now();
        self.sessions.retain(|_, expires_at| *expires_at > now);
    }
}

fn shared_oauth_service<S>(store: S) -> SharedOAuthService
where
    S: OAuthStore + Send + 'static,
{
    Arc::new(Mutex::new(GatewayOAuthService::new(
        Box::new(store) as Box<dyn OAuthStore + Send>
    )))
}

#[derive(Clone, Default)]
struct SharedRefreshLocks {
    locked: Arc<StdMutex<BTreeSet<(String, String)>>>,
}

impl SharedRefreshLocks {
    fn try_acquire(&self, subject: String, upstream_id: String) -> Option<RefreshLockGuard> {
        let key = (subject, upstream_id);
        let mut locked = self.locked.lock().expect("refresh lock");
        if !locked.insert(key.clone()) {
            return None;
        }
        Some(RefreshLockGuard {
            locked: self.locked.clone(),
            key,
        })
    }
}

struct RefreshLockGuard {
    locked: Arc<StdMutex<BTreeSet<(String, String)>>>,
    key: (String, String),
}

impl Drop for RefreshLockGuard {
    fn drop(&mut self) {
        self.locked.lock().expect("refresh lock").remove(&self.key);
    }
}

impl ApiErrorResponse {
    fn bad_request(message: String) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "bad_request",
            message,
        }
    }

    fn bad_gateway(message: String) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            code: "bad_gateway",
            message,
        }
    }

    fn not_found(message: String) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message,
        }
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorEnvelope {
                error: ApiErrorBody {
                    code: self.code.to_string(),
                    message: self.message,
                },
            }),
        )
            .into_response()
    }
}
