use super::*;
use agent_auth::ScopeSet;
use agent_gateway::{ProtectedRouteConfig, ProtectedRouteIndex, ProtectedRouteTarget};
use agent_protocol::{McpServerConfig, McpServerId, McpTransportConfig};
use agent_runtime::McpRuntime;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use std::{collections::BTreeMap, sync::Arc};
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

    let resource = request_json(
        router,
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
}

#[tokio::test]
async fn protected_mcp_router_serves_metadata_and_authorized_json_rpc() {
    let runtime = Arc::new(McpRuntime::start(vec![fixture_config()]).await);
    let router = protected_mcp_router(protected_routes(), runtime);

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
    let router = protected_mcp_router(protected_routes(), runtime);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/syslog")
                .header("host", "mcp.example.test")
                .header("x-forwarded-proto", "https")
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

async fn request_json(router: Router, request: Request<Body>) -> Value {
    let response = router.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK);
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

fn protected_routes() -> ProtectedRouteIndex {
    ProtectedRouteIndex::from_routes(vec![ProtectedRouteConfig {
        name: "syslog".to_string(),
        enabled: true,
        public_host: "mcp.example.test".to_string(),
        public_path: "/syslog".to_string(),
        resource_uri: "https://mcp.example.test/syslog".to_string(),
        authorization_servers: vec!["https://auth.example.test".to_string()],
        required_scopes: ScopeSet::parse("mcp:read").expect("scope"),
        target: ProtectedRouteTarget::UpstreamMcp {
            server_id: McpServerId::new("fixture"),
        },
    }])
    .expect("routes")
}
