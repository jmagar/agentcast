use super::*;
use serde_json::json;

#[test]
fn runtime_snapshot_preserves_tool_metadata() {
    let snapshot = RuntimeCatalogSnapshot {
        server_id: McpServerId::new("local"),
        server_name: "Local".to_string(),
        status: ServerStatus::Healthy,
        tools: vec![RuntimeTool {
            id: McpToolId::new("echo"),
            name: "echo".to_string(),
            title: Some("Echo".to_string()),
            description: Some("Return input".to_string()),
            input_schema: json!({"type": "object"}),
        }],
    };

    assert_eq!(snapshot.server_id.as_str(), "local");
    assert_eq!(snapshot.tools[0].id.as_str(), "echo");
    assert_eq!(snapshot.tools[0].input_schema["type"], "object");
}
