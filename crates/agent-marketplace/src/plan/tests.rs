use super::*;

#[test]
fn install_plan_preview_is_deterministic() {
    let plan = InstallPlan::new("io.modelcontextprotocol/filesystem").step(InstallStep {
        kind: InstallStepKind::AddMcpUpstream,
        description: "Add filesystem MCP upstream".into(),
        target: "mcp.upstreams.filesystem".into(),
        preview: serde_json::json!({"command": "npx"}),
    });

    assert_eq!(plan.package, "io.modelcontextprotocol/filesystem");
    assert_eq!(plan.steps.len(), 1);
    assert_eq!(plan.steps[0].kind, InstallStepKind::AddMcpUpstream);
}

#[test]
fn empty_plan_has_no_apply_steps() {
    let plan = InstallPlan::new("empty");
    assert!(plan.steps.is_empty());
}
