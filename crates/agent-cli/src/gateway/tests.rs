use super::*;
use agent_auth::ScopeSet;
use agent_gateway::{ProtectedRouteConfig, ProtectedRouteIndex};
use agent_protocol::{McpServerConfig, McpServerId, McpToolId, McpTransportConfig, ServerStatus};
use agent_runtime::{RuntimeCatalogSnapshot, RuntimeTool};
use serde_json::json;

#[test]
fn action_rows_are_shaped_from_gateway_catalog() {
    let catalog = GatewayCatalog::from_snapshots(vec![RuntimeCatalogSnapshot {
        server_id: McpServerId::new("git"),
        server_name: "Git".to_string(),
        status: ServerStatus::Healthy,
        tools: vec![RuntimeTool {
            id: McpToolId::new("status"),
            name: "status".to_string(),
            title: Some("Git status".to_string()),
            description: Some("Inspect working tree".to_string()),
            input_schema: json!({}),
            output_schema: None,
            annotations: None,
        }],
        resources: Vec::new(),
        resource_templates: Vec::new(),
        prompts: Vec::new(),
    }]);

    let rows = GatewayCliView::actions(&catalog);

    assert_eq!(rows[0].id, "mcp:git:status");
    assert_eq!(rows[0].name, "Git status");
}

#[test]
fn protected_route_row_does_not_recompute_matching_policy() {
    let index = ProtectedRouteIndex::from_routes(vec![ProtectedRouteConfig {
        name: "syslog".to_string(),
        enabled: true,
        public_host: "mcp.example.test".to_string(),
        public_path: "/syslog".to_string(),
        resource_uri: "https://mcp.example.test/syslog".to_string(),
        authorization_servers: vec!["https://auth.example.test".to_string()],
        required_scopes: ScopeSet::parse("mcp:read").expect("scope"),
        target: ProtectedRouteTarget::UpstreamMcp {
            server_id: McpServerId::new("syslog"),
        },
    }])
    .expect("index");
    let resolved = index.resolve("mcp.example.test", "/syslog").expect("route");

    let row = GatewayCliView::protected_route(resolved);

    assert_eq!(row.name, "syslog");
    assert_eq!(row.target, "upstream_mcp:syslog");
    assert_eq!(row.required_scopes, vec!["mcp:read"]);
}

#[test]
fn oauth_status_row_is_redacted_status_only() {
    let row = GatewayCliView::oauth_status("user-1", "github", OAuthStatus::Connected);

    assert_eq!(row.subject, "user-1");
    assert_eq!(row.upstream_id, "github");
    assert_eq!(row.status, "connected");
}

#[test]
fn renders_human_action_and_search_tables() {
    let actions = vec![GatewayActionRow {
        id: "mcp:fixture:echo".to_string(),
        name: "Echo".to_string(),
        description: Some("Repeat text".to_string()),
    }];
    let search = vec![GatewaySearchRow {
        action_id: "mcp:fixture:echo".to_string(),
        name: "Echo".to_string(),
        score: 100,
        match_kind: "Name".to_string(),
        matched_fields: vec!["Name".to_string()],
        matched_terms: vec!["echo".to_string()],
        summary: Some("Repeat text".to_string()),
        truncated: false,
    }];

    let action_table = GatewayCliView::render_actions_table(&actions);
    let search_table = GatewayCliView::render_search_table(&search);

    assert!(action_table.contains("ACTION ID"));
    assert!(action_table.contains("mcp:fixture:echo"));
    assert!(search_table.contains("SCORE"));
    assert!(search_table.contains("100"));
    assert!(search_table.contains("Repeat text"));
}

#[test]
fn renders_human_status_rows_without_secret_fields() {
    let status = GatewayCliView::oauth_status("user-1", "github", OAuthStatus::Connected);
    let output = GatewayCliView::render_oauth_status(&status);

    assert!(output.contains("SUBJECT"));
    assert!(output.contains("connected"));
    assert!(!output.contains("token"));
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
            env: Default::default(),
        },
        env_keys: Vec::new(),
    }
}

#[tokio::test]
async fn cli_handlers_call_actions_through_shared_gateway_path() {
    let handlers = GatewayCliHandlers::start(vec![fixture_config()]).await;

    let actions = handlers.list_actions();
    let output = handlers
        .call_action(&actions[0].id, json!({"message": "hello"}))
        .await
        .expect("call action");

    assert_eq!(actions[0].id, "mcp:fixture:echo");
    assert_eq!(output["content"][0]["text"], "hello");
}

#[tokio::test]
async fn cli_handlers_read_resources_and_get_prompts() {
    let handlers = GatewayCliHandlers::start(vec![fixture_config()]).await;

    let resource = handlers
        .read_resource("fixture", "fixture://echo")
        .await
        .expect("read resource");
    let prompt = handlers
        .get_prompt(
            "fixture",
            "summarize",
            Some(serde_json::Map::from_iter([(
                "topic".to_string(),
                json!("gateway"),
            )])),
        )
        .await
        .expect("prompt");

    assert_eq!(resource["contents"][0]["text"], "fixture resource");
    assert_eq!(
        prompt["messages"][0]["content"]["text"],
        "Summarize gateway"
    );
}
