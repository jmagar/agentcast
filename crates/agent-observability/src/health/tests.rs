use super::*;

#[test]
fn health_status_has_stable_json_shape() {
    let summary = HealthSummary {
        status: HealthStatus::Degraded,
        component: "gateway".into(),
        message: Some("one upstream down".into()),
    };
    assert_eq!(serde_json::to_value(summary).unwrap()["status"], "degraded");
}
