use super::*;
use agent_protocol::{McpServerConfig, McpServerId, McpTransportConfig};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
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
