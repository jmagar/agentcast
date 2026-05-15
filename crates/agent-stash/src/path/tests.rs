use super::*;

#[test]
fn rejects_absolute_or_parent_paths() {
    assert!(validate_relative_stash_path("prompts/review.md").is_ok());
    assert!(validate_relative_stash_path("/etc/passwd").is_err());
    assert!(validate_relative_stash_path("../secret").is_err());
    assert!(validate_relative_stash_path("prompts\\review.md").is_err());
}

#[test]
fn safe_relative_path_deserializes_only_valid_paths() {
    let path: SafeRelativePath = serde_json::from_str("\"refs/local.md\"").unwrap();
    assert_eq!(path.as_str(), "refs/local.md");

    let invalid = serde_json::from_str::<SafeRelativePath>("\"/tmp/secret.md\"");
    assert!(invalid.is_err());
}
