use super::*;
use std::path::PathBuf;

fn temp_path(name: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "agentcast-ops-{name}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn config_set_creates_file_and_get_reads_it_back() {
    let path = temp_path("set-create", "toml");
    fs::write(
        &path,
        r#"
        [mcp.upstreams.git]
        transport = "stdio"
        command = "git-mcp"
        "#,
    )
    .unwrap();

    let record = config_set(&path, "mcp.upstreams.git.command", "git-mcp-v2").expect("set");
    assert!(record.validation_error.is_none());
    let got = config_get(&path, "mcp.upstreams.git.command").expect("get");
    assert_eq!(got.value, Some(serde_json::json!("git-mcp-v2")));
}

#[test]
fn config_set_allows_partial_upstream_with_validation_warning() {
    let path = temp_path("partial", "toml");
    let record = config_set(&path, "mcp.upstreams.new.transport", "stdio")
        .expect("set succeeds even partial");
    assert!(record.validation_error.is_some());
    let raw = fs::read_to_string(&path).unwrap();
    assert!(raw.contains("transport = \"stdio\""));
}

#[test]
fn config_unset_removes_optional_field() {
    let path = temp_path("unset", "toml");
    fs::write(
        &path,
        r#"
        [mcp.upstreams.fs]
        transport = "stdio"
        command = "fs-mcp"
        args = ["--root", "/tmp"]
        "#,
    )
    .unwrap();

    let record = config_unset(&path, "mcp.upstreams.fs.args").expect("unset");
    assert!(record.removed);
    let after = config_get(&path, "mcp.upstreams.fs.args").expect("get");
    assert!(after.value.is_none());
}

#[test]
fn config_validate_returns_error_for_partial_config() {
    let path = temp_path("validate", "toml");
    config_set(&path, "mcp.upstreams.fs.transport", "stdio").expect("set");
    let err = config_validate(&path).unwrap_err();
    assert_eq!(err.kind(), "parse_error");
}

#[test]
fn env_set_and_unset_roundtrip() {
    let path = temp_path("env", "env");
    let record = env_set(&path, "RUST_LOG", "info").expect("set");
    assert!(!record.backup_written);
    assert_eq!(
        env_get(&path, "RUST_LOG").unwrap().value,
        Some("info".into())
    );

    let unset = env_unset(&path, "RUST_LOG").expect("unset");
    assert!(unset.removed);
    assert_eq!(env_get(&path, "RUST_LOG").unwrap().value, None);
}

#[test]
fn env_list_returns_keys() {
    let path = temp_path("env-list", "env");
    env_set(&path, "A", "1").expect("set");
    env_set(&path, "B", "2").expect("set");
    let listing = env_list(&path).expect("list");
    assert_eq!(listing.keys, vec!["A", "B"]);
}

#[test]
fn config_paths_renders_both_paths() {
    let paths = config_paths(
        std::path::Path::new("/tmp/config.toml"),
        std::path::Path::new("/tmp/.env"),
    );
    assert_eq!(paths.config, "/tmp/config.toml");
    assert_eq!(paths.env_file, "/tmp/.env");
}
