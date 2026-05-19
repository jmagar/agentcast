use super::*;
use agent_protocol::{McpServerConfig, McpTransportConfig};
use std::{collections::BTreeMap, path::PathBuf};

#[test]
fn load_mcp_configs_keeps_imports_disabled_by_default() {
    let path = write_config();
    let configs = load_mcp_configs_from(Some(&path), false, false).expect("configs");

    assert_eq!(configs.len(), 1);
    assert!(!configs[0].enabled);
}

#[test]
fn load_mcp_configs_can_enable_operator_supplied_imports() {
    let path = write_config();
    let configs = load_mcp_configs_from(Some(&path), false, true).expect("configs");

    assert_eq!(configs.len(), 1);
    assert!(configs[0].enabled);
}

#[test]
fn empty_config_without_path_is_valid() {
    let configs = load_mcp_configs_from(None, false, false).expect("configs");
    assert!(configs.is_empty());
}

#[test]
fn server_config_shape_stays_in_protocol_models() {
    let configs = load_mcp_configs_from(None, false, false).expect("configs");
    assert_eq!(configs, Vec::<McpServerConfig>::new());

    let _transport = McpTransportConfig::Stdio {
        command: "node".to_string(),
        args: Vec::new(),
        env: BTreeMap::new(),
    };
}

fn write_config() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "agentcast-server-test-{}-{}.json",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    std::fs::write(
        &path,
        r#"{"mcpServers":{"fixture":{"command":"node","args":["server.js"]}}}"#,
    )
    .expect("write config");
    path
}
