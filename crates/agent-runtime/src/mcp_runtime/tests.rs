use super::*;
use agent_protocol::{McpServerConfig, McpTransportConfig};
use serde_json::json;

fn fixture_config(enabled: bool) -> McpServerConfig {
    let server = format!(
        "{}/../agent-mcp/src/fixtures/mcp_echo_server.js",
        env!("CARGO_MANIFEST_DIR")
    );
    McpServerConfig {
        id: McpServerId::new("fixture"),
        name: "Fixture".to_string(),
        enabled,
        transport: McpTransportConfig::Stdio {
            command: "node".to_string(),
            args: vec![server],
            env: BTreeMap::new(),
        },
        env_keys: Vec::new(),
    }
}

#[tokio::test]
async fn runtime_discovers_enabled_stdio_upstream_catalog() {
    let runtime = McpRuntime::start(vec![fixture_config(true)]).await;
    let snapshot = runtime.snapshots().remove(0);

    assert_eq!(snapshot.status, ServerStatus::Healthy);
    assert_eq!(snapshot.tools[0].name, "echo");
    assert_eq!(snapshot.resources[0].name, "fixture");
    assert_eq!(snapshot.resource_templates[0].name, "fixture-template");
    assert_eq!(snapshot.prompts[0].name, "summarize");
}

#[tokio::test]
async fn runtime_invokes_tool_through_mcp_client() {
    let runtime = McpRuntime::start(vec![fixture_config(true)]).await;

    let response = runtime
        .call_tool(ToolCallRequest {
            server_id: McpServerId::new("fixture"),
            tool_id: McpToolId::new("echo"),
            arguments: json!({"message": "hello"}),
        })
        .await
        .expect("tool call");

    assert_eq!(response.output["content"][0]["text"], "hello");
}

#[tokio::test]
async fn disabled_upstream_is_not_started() {
    let runtime = McpRuntime::start(vec![fixture_config(false)]).await;
    let snapshot = runtime.snapshots().remove(0);

    assert_eq!(snapshot.status, ServerStatus::Disabled);
    assert!(
        runtime
            .call_tool(ToolCallRequest {
                server_id: McpServerId::new("fixture"),
                tool_id: McpToolId::new("echo"),
                arguments: json!({})
            })
            .await
            .is_err()
    );
}
