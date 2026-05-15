use super::*;
use agent_protocol::{
    LauncherActionId, McpServerConfig, McpServerId, McpToolId, McpTransportConfig,
};
use agent_runtime::McpRuntime;
use serde_json::json;
use std::collections::BTreeMap;

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

#[tokio::test]
async fn gateway_invokes_action_through_runtime_route() {
    let runtime = McpRuntime::start(vec![fixture_config()]).await;
    let gateway = GatewayService::from_runtime_snapshots(runtime.snapshots());
    let action_id =
        LauncherActionId::from_server_tool(&McpServerId::new("fixture"), &McpToolId::new("echo"));

    let result = gateway
        .invoke(
            &runtime,
            ToolInvocation {
                action_id,
                arguments: json!({"message": "hello"}),
            },
        )
        .await
        .expect("invoke");

    assert_eq!(result.output["content"][0]["text"], "hello");
}

#[tokio::test]
async fn unknown_action_returns_structured_gateway_error() {
    let runtime = McpRuntime::start(vec![fixture_config()]).await;
    let gateway = GatewayService::from_runtime_snapshots(runtime.snapshots());

    let error = gateway
        .invoke(
            &runtime,
            ToolInvocation {
                action_id: LauncherActionId::new("missing"),
                arguments: json!({}),
            },
        )
        .await
        .expect_err("unknown action");

    assert_eq!(error, GatewayError::UnknownAction("missing".to_string()));
}
