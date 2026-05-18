use super::*;
use agent_auth::{FixtureBearerTokenVerifier, OAuthCredential, ScopeSet};
use agent_gateway::{
    ProtectedRouteCollection, ProtectedRouteConfig, ProtectedRouteIndex, ProtectedRouteTarget,
};
use agent_protocol::{McpServerConfig, McpServerId, McpTransportConfig};
use agent_registry::{InMemoryRegistryCache, RegistryCache};
use agent_runtime::McpRuntime;
use agent_store::{OAuthStore, SqliteOAuthStore};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc};
use tokio::sync::Mutex;
use tower::ServiceExt;

#[tokio::test]
async fn gateway_router_lists_searches_calls_and_reads() {
    let api = GatewayApi::start(vec![fixture_config()]).await;
    let router = gateway_router(api);

    let actions = request_json(
        router.clone(),
        Request::builder()
            .uri("/v1/gateway/actions")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(actions[0]["id"], "mcp:fixture:echo");

    let status = request_json(
        router.clone(),
        Request::builder()
            .uri("/v1/gateway/status")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(status["server_count"], 1);

    let servers = request_json(
        router.clone(),
        Request::builder()
            .uri("/v1/gateway/servers")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(servers[0]["id"], "fixture");

    let search = request_json(
        router.clone(),
        Request::builder()
            .uri("/v1/gateway/search?q=echo&limit=1")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(search[0]["action_id"], "mcp:fixture:echo");

    let call = request_json(
        router.clone(),
        Request::builder()
            .method("POST")
            .uri("/v1/gateway/actions/mcp:fixture:echo/call")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "arguments": { "message": "hello" } }).to_string(),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(call["content"][0]["text"], "hello");

    let resources = request_json(
        router.clone(),
        Request::builder()
            .uri("/v1/gateway/resources?server_id=fixture")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(resources[0]["uri"], "fixture://echo");

    let resource = request_json(
        router.clone(),
        Request::builder()
            .method("POST")
            .uri("/v1/gateway/resources/read")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "server_id": "fixture", "uri": "fixture://echo" }).to_string(),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(resource["contents"][0]["text"], "fixture resource");

    let prompts = request_json(
        router.clone(),
        Request::builder()
            .uri("/v1/gateway/prompts?server_id=fixture")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(prompts[0]["name"], "summarize");

    let prompt = request_json(
        router,
        Request::builder()
            .method("POST")
            .uri("/v1/gateway/prompts/get")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "server_id": "fixture",
                    "name": "summarize",
                    "arguments": { "message": "hello" }
                })
                .to_string(),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(prompt["messages"][0]["content"]["text"], "Summarize topic");
}

#[tokio::test]
async fn registry_router_searches_fixture_servers() {
    let router = registry_router(vec![normalized_registry_server()]);

    let response = request_json(
        router,
        Request::builder()
            .uri("/v1/registry/search?q=file&limit=10")
            .body(Body::empty())
            .expect("request"),
    )
    .await;

    assert_eq!(response[0]["name"], "io.modelcontextprotocol/filesystem");
}

#[tokio::test]
async fn registry_router_can_be_backed_by_cache() {
    let mut cache = InMemoryRegistryCache::default();
    cache.put_fetched(normalized_registry_server(), 1_779_000_000);
    let router = registry_router_from_cache(&cache);

    let response = request_json(
        router,
        Request::builder()
            .uri("/v1/registry/search?q=file&limit=10")
            .body(Body::empty())
            .expect("request"),
    )
    .await;

    assert_eq!(response[0]["name"], "io.modelcontextprotocol/filesystem");
}

#[tokio::test]
async fn marketplace_router_plans_and_applies_mcp_server() {
    let router = marketplace_router();
    let server = normalized_registry_server();

    let plan = request_json(
        router.clone(),
        Request::builder()
            .method("POST")
            .uri("/v1/marketplace/mcp/plan")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&MarketplaceMcpPlanRequest {
                    server: server.clone(),
                })
                .expect("json"),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(plan["steps"][0]["kind"], "verify_runtime");

    let applied = request_json(
        router,
        Request::builder()
            .method("POST")
            .uri("/v1/marketplace/mcp/apply")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&MarketplaceMcpApplyRequest {
                    server,
                    config: None,
                    env_values: BTreeMap::from([(
                        "FILESYSTEM_TOKEN".to_string(),
                        "secret".to_string(),
                    )]),
                })
                .expect("json"),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(
        applied["result"]["added_or_replaced_upstreams"],
        json!(["filesystem"])
    );
    assert_eq!(applied["env_values"]["FILESYSTEM_TOKEN"], "[REDACTED]");
    assert_eq!(
        applied["config"]["mcp"]["upstreams"]["filesystem"]["command"],
        "npx"
    );
}

#[tokio::test]
async fn protected_route_admin_router_cruds_statuses_and_tests_routes() {
    let router = protected_route_admin_router(
        ProtectedRouteCollection::new(vec![protected_route("syslog", "/syslog")]).expect("routes"),
    );

    let list = request_json(
        router.clone(),
        Request::builder()
            .uri("/v1/protected-routes")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(list[0]["name"], "syslog");

    let added = request_json(
        router.clone(),
        Request::builder()
            .method("POST")
            .uri("/v1/protected-routes")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_vec(&protected_route("git", "/git")).expect("json"),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(added["name"], "git");

    let status = request_json(
        router.clone(),
        Request::builder()
            .uri("/v1/protected-routes/git/status")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(status["resolves_public_route"], true);

    let tested = request_json(
        router.clone(),
        Request::builder()
            .method("POST")
            .uri("/v1/protected-routes/test")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({"host":"mcp.example.test","path":"/git/tools/list"}).to_string(),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(tested["matched"], true);
    assert_eq!(tested["route_name"], "git");

    let deleted = request_json(
        router,
        Request::builder()
            .method("DELETE")
            .uri("/v1/protected-routes/syslog")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(deleted["name"], "syslog");
}

#[tokio::test]
async fn protected_mcp_router_serves_metadata_and_authorized_json_rpc() {
    let runtime = Arc::new(McpRuntime::start(vec![fixture_config()]).await);
    let router = protected_mcp_fixture_router(protected_routes(), runtime);

    let metadata = request_json(
        router.clone(),
        Request::builder()
            .uri("/.well-known/oauth-protected-resource/syslog")
            .header("host", "mcp.example.test")
            .header("x-forwarded-proto", "https")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(metadata["resource"], "https://mcp.example.test/syslog");

    let response = request_json(
        router,
        Request::builder()
            .method("POST")
            .uri("/syslog")
            .header("host", "mcp.example.test")
            .header("x-forwarded-proto", "https")
            .header("accept", "application/json")
            .header(
                "authorization",
                "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read",
            )
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": {
                        "name": "echo",
                        "arguments": { "message": "protected" }
                    }
                })
                .to_string(),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(response["result"]["content"][0]["text"], "protected");
}

#[tokio::test]
async fn protected_mcp_router_rejects_missing_bearer_with_challenge() {
    let runtime = Arc::new(McpRuntime::start(vec![fixture_config()]).await);
    let router = protected_mcp_fixture_router(protected_routes(), runtime);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/syslog")
                .header("host", "mcp.example.test")
                .header("x-forwarded-proto", "https")
                .header("accept", "application/json")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "tools/list"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(response.headers().contains_key("www-authenticate"));
}

#[tokio::test]
async fn protected_mcp_router_rejects_unacceptable_response_media() {
    let runtime = Arc::new(McpRuntime::start(vec![fixture_config()]).await);
    let router = protected_mcp_fixture_router(protected_routes(), runtime);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/syslog")
                .header("host", "mcp.example.test")
                .header("x-forwarded-proto", "https")
                .header("accept", "text/html")
                .header(
                    "authorization",
                    "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read",
                )
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "tools/list"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
}

#[tokio::test]
async fn protected_mcp_router_rejects_missing_accept_header() {
    let runtime = Arc::new(McpRuntime::start(vec![fixture_config()]).await);
    let router = protected_mcp_fixture_router(protected_routes(), runtime);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/syslog")
                .header("host", "mcp.example.test")
                .header("x-forwarded-proto", "https")
                .header(
                    "authorization",
                    "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read",
                )
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "tools/list"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
}

#[tokio::test]
async fn protected_mcp_router_rejects_unsupported_protocol_version() {
    let runtime = Arc::new(McpRuntime::start(vec![fixture_config()]).await);
    let router = protected_mcp_fixture_router(protected_routes(), runtime);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/syslog")
                .header("host", "mcp.example.test")
                .header("x-forwarded-proto", "https")
                .header("accept", "application/json")
                .header("mcp-protocol-version", "2024-01-01")
                .header(
                    "authorization",
                    "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read",
                )
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "tools/list"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn protected_mcp_router_accepts_notifications_without_response_body() {
    let runtime = Arc::new(McpRuntime::start(vec![fixture_config()]).await);
    let router = protected_mcp_fixture_router(protected_routes(), runtime);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/syslog")
                .header("host", "mcp.example.test")
                .header("x-forwarded-proto", "https")
                .header("accept", "application/json")
                .header("mcp-protocol-version", "2025-11-25")
                .header(
                    "authorization",
                    "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read",
                )
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "jsonrpc": "2.0",
                        "method": "tools/list"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn protected_mcp_router_initializes_and_tracks_sessions() {
    let runtime = Arc::new(McpRuntime::start(vec![fixture_config()]).await);
    let router = protected_mcp_fixture_router(protected_routes(), runtime);

    let initialize = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/syslog")
                .header("host", "mcp.example.test")
                .header("x-forwarded-proto", "https")
                .header("accept", "application/json")
                .header(
                    "authorization",
                    "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read",
                )
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "initialize",
                        "params": {
                            "protocolVersion": "2025-11-25",
                            "capabilities": {},
                            "clientInfo": { "name": "fixture", "version": "0.0.0" }
                        }
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(initialize.status(), StatusCode::OK);
    let session_id = initialize
        .headers()
        .get(MCP_SESSION_ID)
        .and_then(|value| value.to_str().ok())
        .expect("session id")
        .to_string();
    let initialize_body = to_json(initialize).await;
    assert_eq!(initialize_body["result"]["protocolVersion"], "2025-11-25");

    let tools = request_json(
        router,
        Request::builder()
            .method("POST")
            .uri("/syslog")
            .header("host", "mcp.example.test")
            .header("x-forwarded-proto", "https")
            .header("accept", "application/json")
            .header(MCP_SESSION_ID, session_id)
            .header(
                "authorization",
                "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read",
            )
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "tools/list"
                })
                .to_string(),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(tools["result"]["tools"][0]["name"], "echo");
}

#[tokio::test]
async fn protected_mcp_router_rejects_invalid_origin_and_unknown_session() {
    let runtime = Arc::new(McpRuntime::start(vec![fixture_config()]).await);
    let router = protected_mcp_fixture_router(protected_routes(), runtime);

    let invalid_origin = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/syslog")
                .header("host", "mcp.example.test")
                .header("x-forwarded-proto", "https")
                .header("origin", "https://evil.example.test")
                .header("accept", "application/json")
                .header(
                    "authorization",
                    "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read",
                )
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "tools/list"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(invalid_origin.status(), StatusCode::FORBIDDEN);

    let unknown_session = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/syslog")
                .header("host", "mcp.example.test")
                .header("x-forwarded-proto", "https")
                .header("accept", "application/json")
                .header(MCP_SESSION_ID, "missing-session")
                .header(
                    "authorization",
                    "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read",
                )
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "tools/list"
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(unknown_session.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn protected_mcp_router_allows_sse_and_session_delete() {
    let runtime = Arc::new(McpRuntime::start(vec![fixture_config()]).await);
    let router = protected_mcp_fixture_router(protected_routes(), runtime);

    let get = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/syslog")
                .header("host", "mcp.example.test")
                .header("x-forwarded-proto", "https")
                .header("accept", "text/event-stream")
                .header(
                    "authorization",
                    "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read",
                )
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(get.status(), StatusCode::OK);
    assert_eq!(
        get.headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );
    let sse_session_id = get
        .headers()
        .get(MCP_SESSION_ID)
        .and_then(|value| value.to_str().ok())
        .expect("session id");
    assert!(!sse_session_id.is_empty());
    let body = to_bytes(get.into_body(), usize::MAX)
        .await
        .expect("sse body");
    let body = String::from_utf8(body.to_vec()).expect("utf8");
    assert!(body.contains("id: 1"));
    assert!(body.contains("event: endpoint"));
    assert!(body.contains("data: /syslog"));

    let initialize = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/syslog")
                .header("host", "mcp.example.test")
                .header("x-forwarded-proto", "https")
                .header("accept", "application/json")
                .header(
                    "authorization",
                    "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read",
                )
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "initialize",
                        "params": {
                            "protocolVersion": "2025-11-25",
                            "capabilities": {},
                            "clientInfo": { "name": "fixture", "version": "0.0.0" }
                        }
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    let session_id = initialize
        .headers()
        .get(MCP_SESSION_ID)
        .and_then(|value| value.to_str().ok())
        .expect("session id")
        .to_string();

    let deleted = router
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/syslog")
                .header("host", "mcp.example.test")
                .header("x-forwarded-proto", "https")
                .header(MCP_SESSION_ID, session_id)
                .header(
                    "authorization",
                    "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read",
                )
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn protected_mcp_router_fetches_subject_scoped_upstream_credential() {
    let runtime = Arc::new(McpRuntime::start(vec![fixture_config()]).await);
    let mut store = SqliteOAuthStore::in_memory([8; 32]).expect("store");
    store
        .put_credential(OAuthCredential {
            subject: "user-1".to_string(),
            upstream_id: "fixture".to_string(),
            access_token: "upstream-secret-token".to_string(),
            refresh_token: Some("refresh-secret-token".to_string()),
            scopes: ScopeSet::parse("mcp:read").expect("scope"),
            expires_at_unix: 5000,
            refresh_failed: false,
        })
        .expect("credential");
    let state = ProtectedMcpHttpState {
        api: ProtectedMcpRouteApi::new_with_fixture_verifier(protected_routes()),
        runtime,
        oauth: Some(shared_oauth_service(store)),
        sessions: Arc::new(Mutex::new(McpSessions::default())),
    };

    let token = upstream_access_token_for_request(
        &state,
        &ProtectedMcpJsonRpcRequest {
            host: "mcp.example.test".to_string(),
            path: "/syslog".to_string(),
            public_origin: "https://mcp.example.test".to_string(),
            authorization: Some(
                "Bearer sub=user-1;aud=https://mcp.example.test/syslog;scope=mcp:read".to_string(),
            ),
            body: json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "echo",
                    "arguments": { "message": "protected" }
                }
            }),
        },
    )
    .await
    .expect("token");

    assert_eq!(token.as_deref(), Some("upstream-secret-token"));
}

#[tokio::test]
async fn oauth_router_runs_authorize_callback_status_and_clear_without_exposing_tokens() {
    let router = oauth_router();
    let metadata = oauth_metadata();

    let probe = request_json(
        router.clone(),
        Request::builder()
            .method("POST")
            .uri("/v1/oauth/probe")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "issuer_url": "https://auth.example.test",
                    "metadata": metadata
                })
                .to_string(),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(probe["status"], "disconnected");

    let authorization = request_json(
        router.clone(),
        Request::builder()
            .method("POST")
            .uri("/v1/oauth/authorize")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "issuer_url": "https://auth.example.test",
                    "metadata": oauth_metadata(),
                    "subject": "user-1",
                    "upstream_id": "fixture",
                    "client_id": "client-1",
                    "redirect_uri": "https://agentcast.example.test/oauth/callback",
                    "resource_uri": "https://mcp.example.test/syslog",
                    "state": "state-1",
                    "code_challenge": "challenge",
                    "expires_at_unix": 2000,
                    "protected_resource_scopes": "mcp:read"
                })
                .to_string(),
            ))
            .expect("request"),
    )
    .await;
    assert!(
        authorization["authorization_url"]
            .as_str()
            .expect("authorization url")
            .contains("code_challenge_method=S256")
    );
    assert_eq!(authorization["selected_scopes"][0], "mcp:read");

    let registration = request_json(
        router.clone(),
        Request::builder()
            .method("POST")
            .uri("/v1/oauth/register")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "subject": "user-1",
                    "upstream_id": "fixture",
                    "result": {
                        "subject": "user-1",
                        "upstream_id": "fixture",
                        "client_id": "dynamic-client",
                        "client_secret": "dynamic-secret",
                        "client_id_issued_at_unix": 100,
                        "client_secret_expires_at_unix": 5000
                    }
                })
                .to_string(),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(registration["client_id"], "dynamic-client");
    assert_eq!(registration["has_client_secret"], true);
    assert!(!registration.to_string().contains("dynamic-secret"));

    let callback = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/oauth/callback")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "state": "state-1",
                        "subject": "user-1",
                        "code": "code-1",
                        "now_unix": 1000,
                        "credential": {
                            "subject": "user-1",
                            "upstream_id": "fixture",
                            "access_token": "secret-access-token",
                            "refresh_token": "secret-refresh-token",
                            "scopes": ["mcp:read"],
                            "expires_at_unix": 5000,
                            "refresh_failed": false
                        }
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(callback.status(), StatusCode::NO_CONTENT);

    let status = request_json(
        router.clone(),
        Request::builder()
            .uri("/v1/oauth/status?subject=user-1&upstream_id=fixture&now_unix=1000")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(status["status"], "connected");
    assert!(!status.to_string().contains("secret-access-token"));

    let refreshed = request_json(
        router.clone(),
        Request::builder()
            .method("POST")
            .uri("/v1/oauth/refresh")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "subject": "user-1",
                    "upstream_id": "fixture",
                    "now_unix": 4900,
                    "result": {
                        "access_token": "new-secret-access-token",
                        "refresh_token": "new-secret-refresh-token",
                        "scopes": ["mcp:read"],
                        "expires_at_unix": 8000
                    }
                })
                .to_string(),
            ))
            .expect("request"),
    )
    .await;
    assert_eq!(refreshed["status"], "connected");
    assert!(!refreshed.to_string().contains("new-secret-access-token"));

    let cleared = router
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/oauth/credentials?subject=user-1&upstream_id=fixture")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(cleared.status(), StatusCode::NO_CONTENT);

    let status = request_json(
        router,
        Request::builder()
            .uri("/v1/oauth/status?subject=user-1&upstream_id=fixture&now_unix=1000")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(status["status"], "disconnected");
}

#[tokio::test]
async fn oauth_router_can_use_sqlite_store() {
    let store = SqliteOAuthStore::in_memory([9; 32]).expect("store");
    let router = oauth_router_with_store(store);

    let _authorization = request_json(
        router.clone(),
        Request::builder()
            .method("POST")
            .uri("/v1/oauth/authorize")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "issuer_url": "https://auth.example.test",
                    "metadata": oauth_metadata(),
                    "subject": "user-1",
                    "upstream_id": "fixture",
                    "client_id": "client-1",
                    "redirect_uri": "https://agentcast.example.test/oauth/callback",
                    "resource_uri": "https://mcp.example.test/syslog",
                    "state": "state-1",
                    "code_challenge": "challenge",
                    "expires_at_unix": 2000,
                    "protected_resource_scopes": "mcp:read"
                })
                .to_string(),
            ))
            .expect("request"),
    )
    .await;

    let callback = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/oauth/callback")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "state": "state-1",
                        "subject": "user-1",
                        "code": "code-1",
                        "now_unix": 1000,
                        "credential": {
                            "subject": "user-1",
                            "upstream_id": "fixture",
                            "access_token": "secret-access-token",
                            "refresh_token": "secret-refresh-token",
                            "scopes": ["mcp:read"],
                            "expires_at_unix": 5000,
                            "refresh_failed": false
                        }
                    })
                    .to_string(),
                ))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(callback.status(), StatusCode::NO_CONTENT);

    let status = request_json(
        router,
        Request::builder()
            .uri("/v1/oauth/status?subject=user-1&upstream_id=fixture&now_unix=1000")
            .body(Body::empty())
            .expect("request"),
    )
    .await;
    assert_eq!(status["status"], "connected");
    assert!(!status.to_string().contains("secret-access-token"));
}

#[test]
fn refresh_locks_reject_duplicate_until_guard_drops() {
    let locks = SharedRefreshLocks::default();
    let guard = locks
        .try_acquire("user-1".to_string(), "fixture".to_string())
        .expect("first lock");

    assert!(
        locks
            .try_acquire("user-1".to_string(), "fixture".to_string())
            .is_none()
    );
    assert!(
        locks
            .try_acquire("user-2".to_string(), "fixture".to_string())
            .is_some()
    );

    drop(guard);
    assert!(
        locks
            .try_acquire("user-1".to_string(), "fixture".to_string())
            .is_some()
    );
}

#[tokio::test]
async fn api_errors_use_typed_envelope() {
    let response = ApiErrorResponse::bad_request("scope is invalid".to_string()).into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_json(response).await;
    assert_eq!(body["error"]["code"], "bad_request");
    assert_eq!(body["error"]["message"], "scope is invalid");
}

async fn request_json(router: Router, request: Request<Body>) -> Value {
    let response = router.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    to_json(response).await
}

async fn to_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&body).expect("json")
}

fn fixture_config() -> McpServerConfig {
    let server = format!(
        "{}/../agent-mcp/src/fixtures/mcp_echo_server.js",
        env!("CARGO_MANIFEST_DIR")
    );
    McpServerConfig {
        id: McpServerId::new("fixture"),
        name: "Fixture".to_string(),
        enabled: true,
        transport: McpTransportConfig::Stdio {
            command: "node".to_string(),
            args: vec![server],
            env: BTreeMap::new(),
        },
        env_keys: Vec::new(),
    }
}

fn normalized_registry_server() -> NormalizedMcpServer {
    NormalizedMcpServer {
        name: "io.modelcontextprotocol/filesystem".to_string(),
        title: None,
        description: Some("Filesystem MCP server".to_string()),
        latest_version: Some("0.6.2".to_string()),
        packages: vec![agent_registry::NormalizedMcpPackage {
            registry_type: "npm".to_string(),
            identifier: "@modelcontextprotocol/server-filesystem".to_string(),
            version: Some("0.6.2".to_string()),
            runtime_hint: Some("npx".to_string()),
            transport: Some("stdio".to_string()),
            runtime_arguments: vec![json!("-y")],
            package_arguments: vec![json!("/tmp")],
            environment_variables: vec![agent_registry::NormalizedMcpEnvVar {
                name: "FILESYSTEM_TOKEN".to_string(),
                description: Some("Access token".to_string()),
                is_required: true,
                is_secret: true,
                default: None,
            }],
        }],
        remotes: Vec::new(),
        repository_url: None,
        website_url: None,
        provenance: agent_registry::RegistryProvenance::official_mcp(),
        registry_metadata: agent_registry::NormalizedRegistryMetadata::default(),
    }
}

fn protected_routes() -> ProtectedRouteIndex {
    ProtectedRouteIndex::from_routes(vec![protected_route("syslog", "/syslog")]).expect("routes")
}

fn protected_mcp_fixture_router(routes: ProtectedRouteIndex, runtime: Arc<McpRuntime>) -> Router {
    protected_mcp_router_with_verifier(routes, runtime, Arc::new(FixtureBearerTokenVerifier))
}

fn protected_route(name: &str, public_path: &str) -> ProtectedRouteConfig {
    ProtectedRouteConfig {
        name: name.to_string(),
        enabled: true,
        public_host: "mcp.example.test".to_string(),
        public_path: public_path.to_string(),
        resource_uri: format!("https://mcp.example.test{public_path}"),
        authorization_servers: vec!["https://auth.example.test".to_string()],
        required_scopes: ScopeSet::parse("mcp:read").expect("scope"),
        target: ProtectedRouteTarget::UpstreamMcp {
            server_id: McpServerId::new("fixture"),
        },
    }
}

fn oauth_metadata() -> Value {
    json!({
        "issuer": "https://auth.example.test",
        "authorization_endpoint": "https://auth.example.test/authorize",
        "token_endpoint": "https://auth.example.test/token",
        "code_challenge_methods_supported": ["S256"],
        "scopes_supported": ["mcp:read", "mcp:write"]
    })
}
