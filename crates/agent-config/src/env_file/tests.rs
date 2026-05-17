use super::*;
use std::path::PathBuf;

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "agentcast-env-file-{name}-{}-{}.env",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ))
}

#[test]
fn set_env_value_creates_file_when_missing() {
    let path = temp_path("create");
    let result = set_env_value(&path, "RUST_LOG", "info").expect("set succeeds");
    assert!(result.previous.is_none());
    assert!(!result.backup_written);
    assert_eq!(
        get_env_value(&path, "RUST_LOG").unwrap(),
        Some("info".into())
    );
}

#[test]
fn set_env_value_overwrites_existing() {
    let path = temp_path("overwrite");
    std::fs::write(&path, "RUST_LOG=warn\n").unwrap();

    let result = set_env_value(&path, "RUST_LOG", "debug").expect("set succeeds");
    assert_eq!(result.previous.as_deref(), Some("warn"));
    assert!(result.backup_written);
    assert_eq!(
        get_env_value(&path, "RUST_LOG").unwrap(),
        Some("debug".into())
    );
}

#[test]
fn set_env_value_is_noop_when_value_unchanged() {
    let path = temp_path("noop");
    std::fs::write(&path, "RUST_LOG=info\n").unwrap();

    let result = set_env_value(&path, "RUST_LOG", "info").expect("set succeeds");
    assert!(!result.backup_written);
}

#[test]
fn unset_env_value_removes_key_and_returns_prior() {
    let path = temp_path("unset");
    std::fs::write(&path, "RUST_LOG=info\nOTHER=keep\n").unwrap();

    let prior = unset_env_value(&path, "RUST_LOG").expect("unset succeeds");
    assert_eq!(prior.as_deref(), Some("info"));
    assert!(
        get_env_value(&path, "RUST_LOG")
            .expect("get after unset")
            .is_none()
    );
    assert_eq!(get_env_value(&path, "OTHER").unwrap(), Some("keep".into()));
}

#[test]
fn unset_env_value_is_noop_for_missing_key() {
    let path = temp_path("unset-missing");
    std::fs::write(&path, "RUST_LOG=info\n").unwrap();
    assert!(
        unset_env_value(&path, "NOT_THERE")
            .expect("unset returns")
            .is_none()
    );
}

#[test]
fn list_env_keys_returns_sorted_keys() {
    let path = temp_path("list");
    std::fs::write(&path, "ZED=1\nALPHA=2\nMID=3\n").unwrap();
    let keys = list_env_keys(&path).expect("list");
    assert_eq!(keys, vec!["ALPHA", "MID", "ZED"]);
}

#[test]
fn invalid_keys_are_rejected() {
    let path = temp_path("invalid");
    assert!(set_env_value(&path, "lowercase", "x").is_err());
    assert!(set_env_value(&path, "1LEADING", "x").is_err());
    assert!(set_env_value(&path, "WITH-DASH", "x").is_err());
}

#[test]
fn quoting_round_trips_values_with_special_characters() {
    let path = temp_path("quote");
    set_env_value(&path, "MESSAGE", "hello world").expect("set");
    assert_eq!(
        get_env_value(&path, "MESSAGE").unwrap(),
        Some("hello world".into())
    );
}

#[test]
fn quoting_round_trips_quotes_backslashes_and_newlines() {
    let path = temp_path("escapes");
    let value = "say \"hi\" \\ then\nnext line\twith tab";
    set_env_value(&path, "MESSAGE", value).expect("set");
    assert_eq!(
        get_env_value(&path, "MESSAGE").unwrap().as_deref(),
        Some(value),
    );

    set_env_value(&path, "MESSAGE", value).expect("idempotent rewrite");
    assert_eq!(
        get_env_value(&path, "MESSAGE").unwrap().as_deref(),
        Some(value),
    );
}

#[test]
fn set_env_value_propagates_read_errors_on_existing_path() {
    let dir = std::env::temp_dir().join(format!(
        "agentcast-env-dir-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let err = set_env_value(&dir, "RUST_LOG", "info").unwrap_err();
    assert_eq!(err.kind(), "io_error");
    std::fs::remove_dir_all(&dir).ok();
}
