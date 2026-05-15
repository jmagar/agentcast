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
fn discovery_counts_env_keys_after_jsonc_comment_stripping() {
    let home = temp_home("jsonc-env-count");
    write(
        &home,
        ".gemini/mcp.json",
        r#"{
          /* comment before server map */
          "mcpServers": {
            "fixture": {
              "command": "node",
              "env": {
                "TOKEN": "secret" // trailing comment
              }
            }
          }
        }"#,
    );

    let discovered = discover_known_mcp_configs(&home);

    assert_eq!(discovered[0].env_key_count, 1);
    assert_eq!(discovered[0].config.env_keys, vec!["TOKEN"]);
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

#[test]
fn discovery_scans_additional_json_mcp_clients() {
    let home = temp_home("additional-clients");
    write(
        &home,
        ".config/Claude/claude_desktop_config.json",
        r#"{"mcpServers":{"desktop":{"command":"desktop-mcp"}}}"#,
    );
    write(
        &home,
        ".cursor/mcp.json",
        r#"{"mcpServers":{"cursor":{"command":"cursor-mcp"}}}"#,
    );
    write(
        &home,
        ".config/Code/User/mcp.json",
        r#"{"mcpServers":{"vscode":{"command":"vscode-mcp"}}}"#,
    );
    write(
        &home,
        ".windsurf/mcp.json",
        r#"{"mcpServers":{"windsurf":{"command":"windsurf-mcp"}}}"#,
    );
    write(
        &home,
        ".opencode/mcp.json",
        r#"{"mcpServers":{"opencode":{"command":"opencode-mcp"}}}"#,
    );

    let discovered = discover_known_mcp_configs(&home);
    let sources = discovered
        .iter()
        .map(|server| server.source_client.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        sources,
        vec!["claude-desktop", "cursor", "vscode", "windsurf", "opencode",]
    );
}

#[test]
fn discovery_scans_vscode_insiders_and_antigravity() {
    let home = temp_home("vscode-variants");
    write(
        &home,
        ".config/Code - Insiders/User/mcp.json",
        r#"{"mcpServers":{"insiders":{"command":"insiders-mcp"}}}"#,
    );
    write(
        &home,
        ".config/Antigravity/User/mcp.json",
        r#"{"mcpServers":{"antigravity":{"command":"antigravity-mcp"}}}"#,
    );

    let discovered = discover_known_mcp_configs(&home);
    let names = discovered
        .iter()
        .map(|server| server.config.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(names, vec!["insiders", "antigravity"]);
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
