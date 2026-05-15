use agent_api::{GatewayApi, gateway_router, oauth_router, protected_mcp_router};
use agent_auth::ScopeSet;
use agent_config::parse_mcp_json;
use agent_gateway::{ProtectedRouteConfig, ProtectedRouteIndex, ProtectedRouteTarget};
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
    #[arg(long)]
    protected_mcp_host: Option<String>,
    #[arg(long)]
    protected_mcp_path: Option<String>,
    #[arg(long)]
    protected_mcp_server: Option<String>,
    #[arg(long)]
    protected_mcp_resource: Option<String>,
    #[arg(long = "protected-mcp-auth-server")]
    protected_mcp_auth_servers: Vec<String>,
    #[arg(long, default_value = "mcp:read")]
    protected_mcp_scopes: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let configs = load_mcp_configs(args.mcp_config.as_ref(), args.enable_imported)?;
    let protected_routes = protected_route_index(&args)?;
    let api = GatewayApi::start(configs).await;
    let runtime = api.runtime();
    let mut router = gateway_router(api).merge(oauth_router());
    if let Some(routes) = protected_routes {
        router = router.merge(protected_mcp_router(routes, runtime));
    }
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

fn protected_route_index(args: &Args) -> anyhow::Result<Option<ProtectedRouteIndex>> {
    let any_protected_arg = args.protected_mcp_host.is_some()
        || args.protected_mcp_path.is_some()
        || args.protected_mcp_server.is_some()
        || args.protected_mcp_resource.is_some()
        || !args.protected_mcp_auth_servers.is_empty();
    if !any_protected_arg {
        return Ok(None);
    }

    let host = required_arg(args.protected_mcp_host.as_ref(), "--protected-mcp-host")?;
    let path = required_arg(args.protected_mcp_path.as_ref(), "--protected-mcp-path")?;
    let server = required_arg(args.protected_mcp_server.as_ref(), "--protected-mcp-server")?;
    if args.protected_mcp_auth_servers.is_empty() {
        anyhow::bail!("--protected-mcp-auth-server is required for protected MCP routes");
    }

    let resource_uri = args
        .protected_mcp_resource
        .clone()
        .unwrap_or_else(|| format!("http://{host}{path}"));
    let routes = ProtectedRouteIndex::from_routes(vec![ProtectedRouteConfig {
        name: server.clone(),
        enabled: true,
        public_host: host.clone(),
        public_path: path.clone(),
        resource_uri,
        authorization_servers: args.protected_mcp_auth_servers.clone(),
        required_scopes: ScopeSet::parse(&args.protected_mcp_scopes)?,
        target: ProtectedRouteTarget::UpstreamMcp {
            server_id: agent_protocol::McpServerId::new(server),
        },
    }])?;

    Ok(Some(routes))
}

fn required_arg<'a>(value: Option<&'a String>, name: &str) -> anyhow::Result<&'a String> {
    value.ok_or_else(|| anyhow::anyhow!("{name} is required for protected MCP routes"))
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

    #[test]
    fn protected_route_index_requires_complete_route_flags() {
        let args = Args {
            listen: "127.0.0.1:8787".parse().expect("listen"),
            mcp_config: None,
            enable_imported: false,
            protected_mcp_host: Some("mcp.example.test".to_string()),
            protected_mcp_path: None,
            protected_mcp_server: Some("fixture".to_string()),
            protected_mcp_resource: None,
            protected_mcp_auth_servers: vec!["https://auth.example.test".to_string()],
            protected_mcp_scopes: "mcp:read".to_string(),
        };

        assert!(protected_route_index(&args).is_err());
    }

    #[test]
    fn protected_route_index_builds_generic_upstream_route() {
        let args = Args {
            listen: "127.0.0.1:8787".parse().expect("listen"),
            mcp_config: None,
            enable_imported: false,
            protected_mcp_host: Some("mcp.example.test".to_string()),
            protected_mcp_path: Some("/syslog".to_string()),
            protected_mcp_server: Some("fixture".to_string()),
            protected_mcp_resource: Some("https://mcp.example.test/syslog".to_string()),
            protected_mcp_auth_servers: vec!["https://auth.example.test".to_string()],
            protected_mcp_scopes: "mcp:read".to_string(),
        };

        let routes = protected_route_index(&args)
            .expect("route index")
            .expect("protected routes");
        assert!(routes.resolve("mcp.example.test", "/syslog").is_some());
    }
}
