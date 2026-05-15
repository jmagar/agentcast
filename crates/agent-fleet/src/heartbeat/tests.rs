use super::*;
use crate::{CapabilityKind, CapabilitySummary};

#[test]
fn heartbeat_reports_capabilities_without_lab_policy() {
    let heartbeat = FleetHeartbeat::new("node-1", FleetStatus::Online, "2026-05-15T12:00:00Z")
        .with_capability(
            CapabilitySummary::new(CapabilityKind::McpRuntime, "mcp_stdio").with_version("1"),
        );

    let value = serde_json::to_value(heartbeat).unwrap();

    assert_eq!(value["status"], "online");
    assert_eq!(value["capabilities"][0]["kind"], "mcp_runtime");
    assert_eq!(value["capabilities"][0]["name"], "mcp_stdio");
}

#[test]
fn degraded_nodes_are_still_available_targets() {
    let degraded = FleetHeartbeat::new("node-1", FleetStatus::Degraded, "2026-05-15T12:00:00Z");
    let offline = FleetHeartbeat::new("node-2", FleetStatus::Offline, "2026-05-15T12:00:00Z");

    assert!(degraded.is_available());
    assert!(!offline.is_available());
}
