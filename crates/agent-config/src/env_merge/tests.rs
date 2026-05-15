use super::*;

fn temp_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "agentcast-config-{name}-{}-{}.env",
        std::process::id(),
        uuid_like()
    ))
}

fn uuid_like() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string()
}

#[test]
fn env_merge_is_idempotent_and_writes_backup_on_insert() {
    let path = temp_path("merge");
    std::fs::write(&path, "EXISTING_TOKEN=abc\n").unwrap();

    let result = merge_env_file(
        &path,
        &EnvMerge::new([
            ("EXISTING_TOKEN", "abc"),
            ("REMOTE_URL", "https://example.test"),
        ]),
    )
    .unwrap();

    assert_eq!(result.unchanged, ["EXISTING_TOKEN"]);
    assert_eq!(result.inserted, ["REMOTE_URL"]);
    assert!(result.backup_written);

    let second = merge_env_file(
        &path,
        &EnvMerge::new([
            ("EXISTING_TOKEN", "abc"),
            ("REMOTE_URL", "https://example.test"),
        ]),
    )
    .unwrap();
    assert!(second.inserted.is_empty());
    assert_eq!(second.unchanged, ["EXISTING_TOKEN", "REMOTE_URL"]);
}

#[test]
fn env_merge_rejects_conflicting_existing_value() {
    let path = temp_path("conflict");
    std::fs::write(&path, "API_TOKEN=old\n").unwrap();

    let err = merge_env_file(&path, &EnvMerge::new([("API_TOKEN", "new")])).unwrap_err();
    assert_eq!(err.kind(), "env_conflict");
}

#[test]
fn env_merge_restricts_keys_to_secret_or_runtime_shapes() {
    assert!(validate_env_key_for_merge("ALLOW_WRITE").is_err());
    assert!(validate_env_key_for_merge("SERVICE_TOKEN").is_ok());
    assert!(validate_env_key_for_merge("REMOTE_URL").is_ok());
}
