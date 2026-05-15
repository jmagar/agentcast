use super::*;
use agent_protocol::{McpServerConfig, McpServerId, McpTransportConfig};
use serde_json::json;

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
            env: Default::default(),
        },
        env_keys: Vec::new(),
    }
}

#[tokio::test]
async fn api_lists_searches_and_calls_gateway_actions() {
    let api = GatewayApi::start(vec![fixture_config()]).await;

    let actions = api.list_actions();
    let search = api.search_actions("echo", 5);
    let output = api
        .call_action("mcp:fixture:echo", json!({"message": "hello"}))
        .await
        .expect("call");

    assert_eq!(actions[0].id, "mcp:fixture:echo");
    assert_eq!(search[0].action_id, "mcp:fixture:echo");
    assert_eq!(output["content"][0]["text"], "hello");
}

#[tokio::test]
async fn api_reads_gateway_resource_through_runtime() {
    let api = GatewayApi::start(vec![fixture_config()]).await;

    let output = api
        .read_resource("fixture", "fixture://echo")
        .await
        .expect("resource");

    assert_eq!(output["contents"][0]["text"], "fixture resource");
}
