use super::*;

#[test]
fn empty_policy_allows_all_actions() {
    let action = LauncherActionId::new("mcp:git:status");

    assert!(GatewayExposurePolicy::default().allows(
        &action,
        &McpServerId::new("git"),
        &McpToolId::new("status")
    ));
}

#[test]
fn deny_rules_override_allow_rules() {
    let action = LauncherActionId::new("mcp:git:status");
    let policy = GatewayExposurePolicy::default()
        .allow_server(McpServerId::new("git"))
        .deny_tool(McpToolId::new("status"));

    assert!(!policy.allows(&action, &McpServerId::new("git"), &McpToolId::new("status")));
}

#[test]
fn non_empty_allowlist_hides_unmatched_actions() {
    let policy = GatewayExposurePolicy::default().allow_server(McpServerId::new("git"));

    assert!(policy.allows(
        &LauncherActionId::new("mcp:git:status"),
        &McpServerId::new("git"),
        &McpToolId::new("status")
    ));
    assert!(!policy.allows(
        &LauncherActionId::new("mcp:shell:run"),
        &McpServerId::new("shell"),
        &McpToolId::new("run")
    ));
}
