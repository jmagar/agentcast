use super::*;

#[test]
fn revision_serializes_parent_tracking() {
    let revision = StashRevision::new(
        "rev-2",
        "prompt-1",
        "tighten prompt",
        "2026-05-15T12:00:00Z",
    )
    .with_parent("rev-1");

    let value = serde_json::to_value(revision).unwrap();

    assert_eq!(value["id"], "rev-2");
    assert_eq!(value["parent"], "rev-1");
    assert_eq!(value["item_id"], "prompt-1");
}
