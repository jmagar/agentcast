use super::*;

#[test]
fn stash_metadata_serializes_stable_kind_and_drift_status() {
    let meta = StashItemMeta::new(
        "prompt-1",
        StashItemKind::PromptTemplate,
        SafeRelativePath::new("prompts/review.md").unwrap(),
        "Review prompt",
    )
    .with_tag("review")
    .with_drift_status(DriftStatus::Dirty);

    let value = serde_json::to_value(meta).unwrap();

    assert_eq!(value["kind"], "prompt_template");
    assert_eq!(value["path"], "prompts/review.md");
    assert_eq!(value["drift_status"], "dirty");
    assert_eq!(value["tags"][0], "review");
}
