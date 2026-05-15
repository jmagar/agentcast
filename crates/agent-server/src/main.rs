use agent_api::{GatewayApi, gateway_router};
use agent_config::parse_mcp_json;
use agent_protocol::McpServerConfig;
use clap::Parser;
use std::{net::SocketAddr, path::PathBuf};

#[derive(Debug, Parser)]
#[command(name = "agentcast", version, about = "AgentCast gateway server")]
struct Args {
    #[arg(long, default_value = "127.0.0.1:8787")]
    listen: SocketAddr,
    #[arg(long)]
    mcp_config: Option<PathBuf>,
    #[arg(long)]
    enable_imported: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let configs = load_mcp_configs(args.mcp_config.as_ref(), args.enable_imported)?;
    let api = GatewayApi::start(configs).await;
    let router = gateway_router(api);
    let listener = tokio::net::TcpListener::bind(args.listen).await?;

    tracing::info!(listen = %args.listen, "serving AgentCast gateway API");
    axum::serve(listener, router).await?;
    Ok(())
}

fn load_mcp_configs(
    path: Option<&PathBuf>,
    enable_imported: bool,
) -> anyhow::Result<Vec<McpServerConfig>> {
    let Some(path) = path else {
        return Ok(Vec::new());
    };

    let raw = std::fs::read_to_string(path)?;
    let mut configs = parse_mcp_json(&raw)?;
    if enable_imported {
        for config in &mut configs {
            config.enabled = true;
        }
    }
    Ok(configs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_protocol::McpTransportConfig;
    use std::collections::BTreeMap;

    #[test]
    fn load_mcp_configs_keeps_imports_disabled_by_default() {
        let path = write_config();
        let configs = load_mcp_configs(Some(&path), false).expect("configs");

        assert_eq!(configs.len(), 1);
        assert!(!configs[0].enabled);
    }

    #[test]
    fn load_mcp_configs_can_enable_operator_supplied_imports() {
        let path = write_config();
        let configs = load_mcp_configs(Some(&path), true).expect("configs");

        assert_eq!(configs.len(), 1);
        assert!(configs[0].enabled);
    }

    fn write_config() -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("agentcast-server-test-{}.json", std::process::id()));
        std::fs::write(
            &path,
            r#"{"mcpServers":{"fixture":{"command":"node","args":["server.js"]}}}"#,
        )
        .expect("write config");
        path
    }

    #[test]
    fn empty_config_without_path_is_valid() {
        let configs = load_mcp_configs(None, true).expect("configs");
        assert!(configs.is_empty());
    }

    #[test]
    fn server_config_shape_stays_in_protocol_models() {
        let configs = load_mcp_configs(None, false).expect("configs");
        assert_eq!(configs, Vec::<McpServerConfig>::new());

        let _transport = McpTransportConfig::Stdio {
            command: "node".to_string(),
            args: Vec::new(),
            env: BTreeMap::new(),
        };
    }
}
