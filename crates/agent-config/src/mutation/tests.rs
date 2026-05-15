use super::*;
use crate::{McpTransport, StdioUpstreamConfig, load_from_str};

#[test]
fn add_or_replace_upstream_preserves_unrelated_entries() {
    let mut config = load_from_str(
        r#"
        [mcp.upstreams.git]
        transport = "stdio"
        command = "git-mcp"
        "#,
    )
    .unwrap();
    let upstream = McpUpstreamConfig::new(
        "filesystem",
        McpTransport::Stdio(StdioUpstreamConfig {
            command: "npx".into(),
            args: vec!["@modelcontextprotocol/server-filesystem".into()],
            cwd: None,
            env: Default::default(),
        }),
    )
    .unwrap();

    assert!(
        add_or_replace_upstream(&mut config, upstream)
            .unwrap()
            .is_none()
    );
    assert!(config.mcp.upstreams.contains_key("git"));
    assert!(config.mcp.upstreams.contains_key("filesystem"));
}

#[test]
fn remove_upstream_only_removes_target() {
    let mut config = load_from_str(
        r#"
        [mcp.upstreams.git]
        transport = "stdio"
        command = "git-mcp"
        [mcp.upstreams.fs]
        transport = "stdio"
        command = "fs-mcp"
        "#,
    )
    .unwrap();

    assert!(remove_upstream(&mut config, "git").is_some());
    assert!(!config.mcp.upstreams.contains_key("git"));
    assert!(config.mcp.upstreams.contains_key("fs"));
}
