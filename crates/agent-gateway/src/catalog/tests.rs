use super::*;
use agent_protocol::{McpServerId, McpToolId, ServerStatus};
use agent_runtime::{RuntimeCatalogSnapshot, RuntimeTool};
use serde_json::json;

#[test]
fn projects_runtime_tools_to_stable_actions() {
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

    let catalog = GatewayCatalog::from_snapshots(vec![snapshot]);

    assert_eq!(catalog.actions.len(), 1);
    assert_eq!(catalog.actions[0].id.as_str(), "mcp:local:echo");
    assert_eq!(catalog.actions[0].display_name, "Echo");
    assert!(catalog.collisions.is_empty());
}

#[test]
fn reports_duplicate_action_ids() {
    let snapshots = vec![
        RuntimeCatalogSnapshot {
            server_id: McpServerId::new("local"),
            server_name: "Local".to_string(),
            status: ServerStatus::Healthy,
            tools: vec![RuntimeTool {
                id: McpToolId::new("echo"),
                name: "echo".to_string(),
                title: None,
                description: None,
                input_schema: json!({}),
            }],
        },
        RuntimeCatalogSnapshot {
            server_id: McpServerId::new("local"),
            server_name: "Local Duplicate".to_string(),
            status: ServerStatus::Healthy,
            tools: vec![RuntimeTool {
                id: McpToolId::new("echo"),
                name: "echo-alt".to_string(),
                title: None,
                description: None,
                input_schema: json!({}),
            }],
        },
    ];

    let catalog = GatewayCatalog::from_snapshots(snapshots);

    assert_eq!(catalog.actions.len(), 1);
    assert_eq!(catalog.collisions.len(), 1);
    assert_eq!(catalog.collisions[0].action_id.as_str(), "mcp:local:echo");
}
