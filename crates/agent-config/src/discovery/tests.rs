use super::*;
use agent_protocol::McpTransportConfig;

#[test]
fn discover_known_configs_scans_stable_order_and_dedupes_first_seen() {
    let home = temp_home("stable-order");
    write(
        &home,
        ".claude/settings.json",
        r#"{"mcpServers":{"shared":{"command":"claude-mcp"}}}"#,
    );
    write(
        &home,
        ".codex/config.toml",
        r#"[mcp_servers.shared]
command = "codex-mcp"
"#,
    );

    let discovered = discover_known_mcp_configs(&home);

    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].source_client, "claude-code");
    match &discovered[0].config.transport {
        McpTransportConfig::Stdio { command, .. } => assert_eq!(command, "claude-mcp"),
        McpTransportConfig::StreamableHttp { .. } => panic!("expected stdio"),
    }
}

#[test]
fn claude_settings_does_not_use_root_fallback_but_legacy_file_does() {
    let home = temp_home("claude-root-fallback");
    write(
        &home,
        ".claude/settings.json",
        r#"{"root":{"command":"ignored"}}"#,
    );
    write(&home, ".claude.json", r#"{"legacy":{"command":"used"}}"#);

    let discovered = discover_known_mcp_configs(&home);

    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].config.name, "legacy");
}

#[test]
fn discovery_supports_command_arrays_and_scrubs_raw_env_values() {
    let home = temp_home("command-array-env");
    write(
        &home,
        ".gemini/mcp.json",
        r#"{
          "mcpServers": {
            "fixture": {
              "command": ["node", "server.js"],
              "env": {"SECRET_TOKEN": "secret"}
            }
          }
        }"#,
    );

    let discovered = discover_known_mcp_configs(&home);

    assert_eq!(discovered[0].env_key_count, 1);
    assert_eq!(discovered[0].config.env_keys, vec!["SECRET_TOKEN"]);
    match &discovered[0].config.transport {
        McpTransportConfig::Stdio { args, env, .. } => {
            assert_eq!(args, &vec!["server.js".to_string()]);
            assert!(env.is_empty());
        }
        McpTransportConfig::StreamableHttp { .. } => panic!("expected stdio"),
    }
}

#[test]
fn codex_toml_mcp_servers_are_discovered() {
    let home = temp_home("codex");
    write(
        &home,
        ".codex/config.toml",
        r#"[mcp_servers.fixture]
command = "node"
args = ["server.js"]
"#,
    );

    let discovered = discover_known_mcp_configs(&home);

    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].source_client, "codex");
    assert_eq!(discovered[0].config.name, "fixture");
}

fn temp_home(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "agentcast-config-discovery-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("temp home");
    path
}

fn write(home: &Path, rel: &str, content: &str) {
    let path = home.join(rel);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("parent dir");
    std::fs::write(path, content).expect("write");
}
