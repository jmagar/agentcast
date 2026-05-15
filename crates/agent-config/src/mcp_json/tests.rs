use super::*;

#[test]
fn imports_stdio_servers_disabled_by_default() {
    let raw = r#"
    {
      "mcpServers": {
        "local": {
          "command": "agent",
          "args": ["serve"],
          "env": {"TOKEN": "secret"}
        }
      }
    }
    "#;

    let servers = parse_mcp_json(raw).expect("parse config");

    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "local");
    assert!(!servers[0].enabled);
    assert_eq!(servers[0].env_keys, vec!["TOKEN"]);
}

#[test]
fn jsonc_comments_do_not_strip_url_slashes_inside_strings() {
    let raw = r#"
    {
      // comment outside strings
      "mcpServers": {
        "remote": {
          "url": "https://example.test/mcp//stable"
        }
      }
    }
    "#;

    let servers = parse_mcp_json(raw).expect("parse config");

    match &servers[0].transport {
        agent_protocol::McpTransportConfig::StreamableHttp { url, .. } => {
            assert_eq!(url, "https://example.test/mcp//stable");
        }
        agent_protocol::McpTransportConfig::Stdio { .. } => panic!("expected streamable http"),
    }
}

#[test]
fn jsonc_block_comments_are_removed_without_touching_strings() {
    let raw = r#"
    {
      /* block comment with "quoted" text */
      "mcpServers": {
        "remote": {
          "url": "https://example.test/mcp/*literal*/stable"
        }
      }
    }
    "#;

    let servers = parse_mcp_json(raw).expect("parse config");

    match &servers[0].transport {
        agent_protocol::McpTransportConfig::StreamableHttp { url, .. } => {
            assert_eq!(url, "https://example.test/mcp/*literal*/stable");
        }
        agent_protocol::McpTransportConfig::Stdio { .. } => panic!("expected streamable http"),
    }
}

#[test]
fn mcp_servers_object_wins_over_servers_object() {
    let raw = r#"
    {
      "mcpServers": {"preferred": {"command": "preferred"}},
      "servers": {"ignored": {"command": "ignored"}}
    }
    "#;

    let servers = parse_mcp_json(raw).expect("parse config");

    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "preferred");
}
