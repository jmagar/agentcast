use super::*;
use agent_protocol::{LauncherActionId, McpServerId, McpToolId};

#[test]
fn resolves_known_action_to_upstream_tool() {
    let route = ActionRoute {
        action_id: LauncherActionId::from_server_tool(
            &McpServerId::new("local"),
            &McpToolId::new("echo"),
        ),
        server_id: McpServerId::new("local"),
        tool_id: McpToolId::new("echo"),
    };
    let router = GatewayRouter::new(vec![route.clone()]);

    assert_eq!(router.resolve(&route.action_id), Some(&route));
}

#[test]
fn unknown_action_returns_none() {
    let router = GatewayRouter::new(Vec::new());
    let action_id =
        LauncherActionId::from_server_tool(&McpServerId::new("missing"), &McpToolId::new("echo"));

    assert!(router.resolve(&action_id).is_none());
}
