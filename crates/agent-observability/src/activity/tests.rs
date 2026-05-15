use super::*;

#[test]
fn activity_event_serializes_stable_kind() {
    let event = ActivityEvent {
        kind: ActivityKind::InstallPlan,
        target: "filesystem".into(),
        message: "planned install".into(),
        metadata: serde_json::json!({"steps": 2}),
    };

    let value = serde_json::to_value(event).unwrap();
    assert_eq!(value["kind"], "install_plan");
}
