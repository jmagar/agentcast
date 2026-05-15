use super::*;
use crate::{EnvBinding, McpTransport, StdioUpstreamConfig};

#[test]
fn loads_stdio_mcp_upstream_with_structured_command() {
    let config = load_from_str(
        r#"
        [mcp.upstreams.filesystem]
        transport = "stdio"
        command = "npx"
        args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
        cwd = "/tmp"

        [mcp.upstreams.filesystem.env.ALLOW_WRITE]
        source = "config"
        value = "false"
        "#,
    )
    .expect("config parses");

    let upstream = config
        .mcp
        .upstreams
        .get("filesystem")
        .expect("upstream exists");
    assert_eq!(upstream.id, "filesystem");

    let McpTransport::Stdio(StdioUpstreamConfig {
        command,
        args,
        cwd,
        env,
    }) = &upstream.transport
    else {
        panic!("expected stdio upstream");
    };
    assert_eq!(command, "npx");
    assert_eq!(
        args,
        &["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    );
    assert_eq!(cwd.as_deref(), Some(std::path::Path::new("/tmp")));
    assert_eq!(
        env.get("ALLOW_WRITE"),
        Some(&EnvBinding::Config {
            value: "false".into()
        })
    );
}

#[test]
fn rejects_blank_upstream_id() {
    let err = load_from_str(
        r#"
        [mcp.upstreams.""]
        transport = "stdio"
        command = "node"
        "#,
    )
    .expect_err("blank upstream id is invalid");

    assert_eq!(err.kind(), "invalid_config");
    assert!(err.to_string().contains("upstream id"));
}
