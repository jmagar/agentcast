use super::*;

#[test]
fn action_id_is_derived_from_server_and_tool() {
    let server_id = McpServerId::new("local");
    let tool_id = McpToolId::new("echo");

    let action_id = LauncherActionId::from_server_tool(&server_id, &tool_id);

    assert_eq!(action_id.as_str(), "mcp:local:echo");
}

#[test]
fn ids_display_inner_value() {
    assert_eq!(McpServerId::new("server").to_string(), "server");
    assert_eq!(McpToolId::new("tool").to_string(), "tool");
    assert_eq!(
        LauncherActionId::from_server_tool(&McpServerId::new("s"), &McpToolId::new("t"))
            .to_string(),
        "mcp:s:t"
    );
}
