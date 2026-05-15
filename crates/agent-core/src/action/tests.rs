use super::*;

#[test]
fn action_metadata_serializes_stable_kind_strings() {
    let metadata = ActionMetadata {
        action_id: "mcp:filesystem:tool:read_file".into(),
        title: "Read file".into(),
        description: Some("Read a local file".into()),
        category: Some(ActionCategory::Filesystem),
        risk: ActionRisk::ReadOnly,
    };

    let value = serde_json::to_value(metadata).unwrap();
    assert_eq!(value["category"], "filesystem");
    assert_eq!(value["risk"], "read_only");
}
