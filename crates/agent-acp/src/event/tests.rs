use super::*;

#[test]
fn acp_event_preserves_unknown_payload() {
    let event = AcpEvent {
        kind: AcpEventKind::Unknown,
        payload: serde_json::json!({"provider": "custom", "raw": true}),
    };
    let value = serde_json::to_value(event).unwrap();
    assert_eq!(value["kind"], "unknown");
    assert_eq!(value["payload"]["raw"], true);
}
