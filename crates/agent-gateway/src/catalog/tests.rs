use super::*;
use agent_protocol::{McpServerId, McpToolId, ServerStatus};
use agent_runtime::{RuntimeCatalogSnapshot, RuntimeTool};
use serde_json::{Value, json};

fn tool(
    id: &str,
    name: &str,
    title: Option<&str>,
    description: Option<&str>,
    input_schema: Value,
) -> RuntimeTool {
    RuntimeTool {
        id: McpToolId::new(id),
        name: name.to_string(),
        title: title.map(str::to_string),
        description: description.map(str::to_string),
        input_schema,
        output_schema: None,
        annotations: None,
    }
}

fn snapshot(server_id: &str, server_name: &str, tools: Vec<RuntimeTool>) -> RuntimeCatalogSnapshot {
    RuntimeCatalogSnapshot {
        server_id: McpServerId::new(server_id),
        server_name: server_name.to_string(),
        status: ServerStatus::Healthy,
        tools,
        resources: Vec::new(),
        resource_templates: Vec::new(),
        prompts: Vec::new(),
    }
}

#[test]
fn projects_runtime_tools_to_stable_actions() {
    let catalog = GatewayCatalog::from_snapshots(vec![snapshot(
        "local",
        "Local",
        vec![tool(
            "echo",
            "echo",
            Some("Echo"),
            Some("Return input"),
            json!({"type": "object"}),
        )],
    )]);

    assert_eq!(catalog.actions.len(), 1);
    assert_eq!(catalog.actions[0].id.as_str(), "mcp:local:echo");
    assert_eq!(catalog.actions[0].display_name, "Echo");
    assert!(catalog.collisions.is_empty());
}

#[test]
fn reports_duplicate_action_ids() {
    let snapshots = vec![
        snapshot(
            "local",
            "Local",
            vec![tool("echo", "echo", None, None, json!({}))],
        ),
        snapshot(
            "local",
            "Local Duplicate",
            vec![tool("echo", "echo-alt", None, None, json!({}))],
        ),
    ];

    let catalog = GatewayCatalog::from_snapshots(snapshots);

    assert_eq!(catalog.actions.len(), 1);
    assert_eq!(catalog.collisions.len(), 1);
    assert_eq!(catalog.collisions[0].action_id.as_str(), "mcp:local:echo");
}

#[test]
fn catalog_exports_search_documents_without_ranking() {
    let catalog = GatewayCatalog::from_snapshots(vec![snapshot(
        "git",
        "Git",
        vec![tool(
            "status",
            "status",
            Some("Git status"),
            Some("Inspect working tree"),
            json!({"type": "object"}),
        )],
    )]);

    let docs = catalog.search_documents();

    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].action_id.as_str(), "mcp:git:status");
    assert_eq!(docs[0].name, "Git status");
    assert_eq!(docs[0].description.as_deref(), Some("Inspect working tree"));
    assert!(docs[0].metadata.contains(&"git".to_string()));
    assert!(!docs[0].catalog_hash.is_empty());
}

#[test]
fn exposure_policy_filters_actions_before_search_and_routing() {
    let catalog = GatewayCatalog::from_snapshots_with_policy(
        vec![snapshot(
            "git",
            "Git",
            vec![
                tool(
                    "status",
                    "status",
                    Some("Git status"),
                    Some("Inspect working tree"),
                    json!({"type": "object"}),
                ),
                tool(
                    "push",
                    "push",
                    Some("Git push"),
                    Some("Push commits"),
                    json!({"type": "object"}),
                ),
            ],
        )],
        &GatewayExposurePolicy::default().deny_tool(McpToolId::new("push")),
    );

    assert_eq!(
        catalog
            .actions
            .iter()
            .map(|action| action.id.as_str())
            .collect::<Vec<_>>(),
        ["mcp:git:status"]
    );
    assert_eq!(catalog.search_documents().len(), 1);
}
