use super::*;

#[test]
fn gateway_action_view_has_stable_json() {
    let view = GatewayActionView {
        id: "mcp:fixture:tool:echo".into(),
        name: "echo".into(),
        description: None,
        server_id: Some("fixture".into()),
    };
    assert_eq!(serde_json::to_value(view).unwrap()["server_id"], "fixture");
}
