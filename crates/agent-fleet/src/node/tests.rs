use super::*;

#[test]
fn fleet_node_serializes_generic_identity() {
    let node = FleetNode::new("node-1", "Workshop").with_label("os", "linux");

    let value = serde_json::to_value(node).unwrap();

    assert_eq!(value["node_id"], "node-1");
    assert_eq!(value["display_name"], "Workshop");
    assert_eq!(value["labels"]["os"], "linux");
}
