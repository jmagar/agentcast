use super::*;
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};

#[derive(Clone)]
struct FixtureBackend;

#[async_trait]
impl GatewayMcpBackend for FixtureBackend {
    fn gateway_status(&self) -> GatewayMcpStatus {
        GatewayMcpStatus {
            server_count: 1,
            action_count: 1,
        }
    }

    fn list_servers(&self) -> Vec<GatewayMcpServer> {
        vec![GatewayMcpServer {
            id: "fixture".to_string(),
            name: "Fixture".to_string(),
            status: "running".to_string(),
            tool_count: 1,
            resource_count: 1,
            prompt_count: 1,
        }]
    }

    fn list_actions(&self) -> Vec<GatewayMcpAction> {
        vec![GatewayMcpAction {
            id: "mcp:fixture:echo".to_string(),
            name: "echo".to_string(),
            description: Some("Echo input".to_string()),
        }]
    }

    fn search_actions(&self, query: &str, limit: usize) -> Vec<GatewayMcpSearchResult> {
        assert_eq!(query, "echo");
        assert_eq!(limit, 1);
        vec![GatewayMcpSearchResult {
            action_id: "mcp:fixture:echo".to_string(),
            name: "echo".to_string(),
            score: 100,
            match_kind: "Exact".to_string(),
            truncated: false,
        }]
    }

    async fn call_action(&self, action_id: &str, arguments: Value) -> Result<Value, String> {
        assert_eq!(action_id, "mcp:fixture:echo");
        Ok(json!({ "called": action_id, "arguments": arguments }))
    }

    fn list_resources(&self, server_id: Option<&str>) -> Vec<GatewayMcpResource> {
        assert_eq!(server_id, Some("fixture"));
        vec![GatewayMcpResource {
            server_id: "fixture".to_string(),
            uri: "fixture://echo".to_string(),
            name: "echo-resource".to_string(),
            title: Some("Echo resource".to_string()),
            description: None,
            mime_type: Some("text/plain".to_string()),
        }]
    }

    async fn read_resource(&self, server_id: &str, uri: &str) -> Result<Value, String> {
        assert_eq!(server_id, "fixture");
        assert_eq!(uri, "fixture://echo");
        Ok(json!({ "contents": [{ "uri": uri, "text": "fixture resource" }] }))
    }

    fn list_prompts(&self, server_id: Option<&str>) -> Vec<GatewayMcpPrompt> {
        assert_eq!(server_id, Some("fixture"));
        vec![GatewayMcpPrompt {
            server_id: "fixture".to_string(),
            name: "echo_prompt".to_string(),
            title: Some("Echo prompt".to_string()),
            description: None,
            arguments: json!({}),
        }]
    }

    async fn get_prompt(
        &self,
        server_id: &str,
        name: &str,
        arguments: Option<JsonObject>,
    ) -> Result<Value, String> {
        assert_eq!(server_id, "fixture");
        assert_eq!(name, "echo_prompt");
        assert_eq!(
            arguments
                .as_ref()
                .and_then(|args| args.get("message"))
                .and_then(Value::as_str),
            Some("hi")
        );
        Ok(json!({ "messages": [{ "role": "user", "content": { "type": "text", "text": "hi" } }] }))
    }
}

#[test]
fn gateway_mcp_server_exposes_expected_tools() {
    let tools = AgentCastMcpServer::<FixtureBackend>::tools();
    let names = tools
        .iter()
        .map(|tool| tool.name.as_ref())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            "agentcast_gateway_list_servers",
            "agentcast_gateway_list_actions",
            "agentcast_gateway_search_actions",
            "agentcast_gateway_call_action",
            "agentcast_gateway_list_resources",
            "agentcast_gateway_read_resource",
            "agentcast_gateway_list_prompts",
            "agentcast_gateway_get_prompt",
            "agentcast_gateway_status",
        ]
    );
    assert!(
        tools
            .iter()
            .all(|tool| tool.input_schema.get("type").is_some())
    );
}

#[tokio::test]
async fn gateway_mcp_server_delegates_tool_calls_to_backend() {
    let server = AgentCastMcpServer::new(FixtureBackend);

    let listed = server
        .call_gateway_tool(CallToolRequestParams::new("agentcast_gateway_list_actions"))
        .await
        .expect("list actions");
    assert_eq!(listed[0]["id"], "mcp:fixture:echo");

    let searched = server
        .call_gateway_tool(
            CallToolRequestParams::new("agentcast_gateway_search_actions").with_arguments(
                serde_json::from_value(json!({ "q": "echo", "limit": 1 })).expect("arguments"),
            ),
        )
        .await
        .expect("search actions");
    assert_eq!(searched[0]["score"], 100);

    let called = server
        .call_gateway_tool(
            CallToolRequestParams::new("agentcast_gateway_call_action").with_arguments(
                serde_json::from_value(json!({
                    "action_id": "mcp:fixture:echo",
                    "arguments": { "message": "hi" }
                }))
                .expect("arguments"),
            ),
        )
        .await
        .expect("call action");
    assert_eq!(called["arguments"]["message"], "hi");

    let servers = server
        .call_gateway_tool(CallToolRequestParams::new("agentcast_gateway_list_servers"))
        .await
        .expect("list servers");
    assert_eq!(servers[0]["id"], "fixture");

    let resources = server
        .call_gateway_tool(
            CallToolRequestParams::new("agentcast_gateway_list_resources").with_arguments(
                serde_json::from_value(json!({ "server_id": "fixture" })).expect("arguments"),
            ),
        )
        .await
        .expect("list resources");
    assert_eq!(resources[0]["uri"], "fixture://echo");

    let resource = server
        .call_gateway_tool(
            CallToolRequestParams::new("agentcast_gateway_read_resource").with_arguments(
                serde_json::from_value(json!({
                    "server_id": "fixture",
                    "uri": "fixture://echo"
                }))
                .expect("arguments"),
            ),
        )
        .await
        .expect("read resource");
    assert_eq!(resource["contents"][0]["text"], "fixture resource");

    let prompts = server
        .call_gateway_tool(
            CallToolRequestParams::new("agentcast_gateway_list_prompts").with_arguments(
                serde_json::from_value(json!({ "server_id": "fixture" })).expect("arguments"),
            ),
        )
        .await
        .expect("list prompts");
    assert_eq!(prompts[0]["name"], "echo_prompt");

    let prompt = server
        .call_gateway_tool(
            CallToolRequestParams::new("agentcast_gateway_get_prompt").with_arguments(
                serde_json::from_value(json!({
                    "server_id": "fixture",
                    "name": "echo_prompt",
                    "arguments": { "message": "hi" }
                }))
                .expect("arguments"),
            ),
        )
        .await
        .expect("get prompt");
    assert_eq!(prompt["messages"][0]["content"]["text"], "hi");

    let status = server
        .call_gateway_tool(CallToolRequestParams::new("agentcast_gateway_status"))
        .await
        .expect("status");
    assert_eq!(status["server_count"], 1);
}
