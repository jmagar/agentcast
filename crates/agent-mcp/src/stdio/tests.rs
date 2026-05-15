use super::*;
use crate::{McpClient, McpClientOptions, McpResourceContent};
use serde_json::json;

fn fixture_connection() -> StdioConnection {
    let server = std::env::current_dir()
        .expect("current dir")
        .join("src/fixtures/mcp_echo_server.js");
    StdioConnection::new("node", [server.display().to_string()])
}

#[tokio::test]
async fn stdio_client_lists_tools_from_fixture_server() {
    let client = McpClient::connect_stdio(fixture_connection(), McpClientOptions::default())
        .await
        .expect("fixture client connects");

    let tools = client.list_tools().await.expect("tools listed");

    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "echo");
}

#[tokio::test]
async fn stdio_client_calls_fixture_tool() {
    let client = McpClient::connect_stdio(fixture_connection(), McpClientOptions::default())
        .await
        .expect("fixture client connects");

    let result = client
        .call_tool("echo", json!({"message": "hello"}))
        .await
        .expect("tool call succeeds");

    assert_eq!(result.text(), Some("hello"));
}

#[tokio::test]
async fn stdio_client_lists_and_reads_resources() {
    let client = McpClient::connect_stdio(fixture_connection(), McpClientOptions::default())
        .await
        .expect("fixture client connects");

    let resources = client.list_resources().await.expect("resources listed");
    let content = client
        .read_resource(&resources[0].uri)
        .await
        .expect("resource read");

    assert_eq!(resources[0].name, "fixture");
    assert_eq!(
        content.contents[0],
        McpResourceContent::Text {
            uri: "fixture://echo".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: "fixture resource".to_string()
        }
    );
}

#[tokio::test]
async fn stdio_client_lists_and_gets_prompts() {
    let client = McpClient::connect_stdio(fixture_connection(), McpClientOptions::default())
        .await
        .expect("fixture client connects");

    let prompts = client.list_prompts().await.expect("prompts listed");
    let prompt = client
        .get_prompt("summarize", Some(serde_json::Map::from_iter([("topic".to_string(), json!("gateway"))])))
        .await
        .expect("prompt");

    assert_eq!(prompts[0].name, "summarize");
    assert_eq!(prompt["messages"][0]["content"]["text"], "Summarize gateway");
}
