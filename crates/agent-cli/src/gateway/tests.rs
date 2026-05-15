use super::*;
use agent_auth::ScopeSet;
use agent_gateway::{ProtectedRouteConfig, ProtectedRouteIndex};
use agent_protocol::{McpServerId, McpToolId, ServerStatus};
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
        }],
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
