use super::*;

fn fixture() -> TomlValue {
    let raw = r#"
        [mcp.upstreams.git]
        transport = "stdio"
        command = "git-mcp"
        enabled = true

        [mcp.upstreams.fs]
        transport = "stdio"
        command = "fs-mcp"
        args = ["--root", "/tmp"]
    "#;
    toml::from_str(raw).expect("fixture parses as toml::Value")
}

#[test]
fn get_value_reads_nested_scalar() {
    let root = fixture();
    let value = get_value(&root, "mcp.upstreams.git.command")
        .expect("get succeeds")
        .expect("value present");
    assert_eq!(value.as_str(), Some("git-mcp"));
}

#[test]
fn get_value_returns_none_for_unknown_path() {
    let root = fixture();
    assert!(
        get_value(&root, "mcp.upstreams.missing.command")
            .expect("get succeeds")
            .is_none()
    );
}

#[test]
fn set_value_updates_scalar_string() {
    let mut root = fixture();
    set_value(&mut root, "mcp.upstreams.git.command", "git-mcp-2").expect("set");
    assert_eq!(
        get_value(&root, "mcp.upstreams.git.command")
            .unwrap()
            .unwrap()
            .as_str(),
        Some("git-mcp-2")
    );
}

#[test]
fn set_value_parses_json_typed_inputs() {
    let mut root = fixture();
    set_value(&mut root, "mcp.upstreams.git.enabled", "false").expect("bool parses");
    set_value(
        &mut root,
        "mcp.upstreams.git.args",
        r#"["--repo","/srv/git"]"#,
    )
    .expect("array parses");

    assert_eq!(
        get_value(&root, "mcp.upstreams.git.enabled")
            .unwrap()
            .unwrap()
            .as_bool(),
        Some(false)
    );
    let args = get_value(&root, "mcp.upstreams.git.args").unwrap().unwrap();
    let items = args.as_array().expect("array");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].as_str(), Some("--repo"));
}

#[test]
fn set_value_rejects_null() {
    let mut root = fixture();
    let err = set_value(&mut root, "mcp.upstreams.git.command", "null").unwrap_err();
    assert_eq!(err.kind(), "invalid_config");
}

#[test]
fn set_value_creates_intermediate_tables() {
    let mut root = TomlValue::Table(Table::new());
    set_value(&mut root, "mcp.upstreams.new.transport", "stdio").expect("set");
    let nested = get_value(&root, "mcp.upstreams.new.transport")
        .unwrap()
        .unwrap();
    assert_eq!(nested.as_str(), Some("stdio"));
}

#[test]
fn unset_value_removes_existing_field() {
    let mut root = fixture();
    assert!(unset_value(&mut root, "mcp.upstreams.fs.args").expect("unset succeeds"));
    assert!(get_value(&root, "mcp.upstreams.fs.args").unwrap().is_none());
}

#[test]
fn unset_value_returns_false_for_missing_key() {
    let mut root = fixture();
    assert!(!unset_value(&mut root, "mcp.upstreams.fs.cwd").expect("unset succeeds"));
}

#[test]
fn empty_path_is_rejected() {
    let root = fixture();
    assert!(get_value(&root, "").is_err());
    assert!(get_value(&root, "mcp..upstreams").is_err());
}

#[test]
fn validate_as_agent_config_succeeds_for_complete_upstream() {
    let mut root = TomlValue::Table(Table::new());
    set_value(&mut root, "mcp.upstreams.git.transport", "stdio").unwrap();
    set_value(&mut root, "mcp.upstreams.git.command", "git-mcp").unwrap();
    let config = validate_as_agent_config(&root).expect("validates");
    assert!(config.mcp.upstreams.contains_key("git"));
}

#[test]
fn validate_as_agent_config_fails_when_partial() {
    let mut root = TomlValue::Table(Table::new());
    set_value(&mut root, "mcp.upstreams.git.transport", "stdio").unwrap();
    assert!(validate_as_agent_config(&root).is_err());
}
