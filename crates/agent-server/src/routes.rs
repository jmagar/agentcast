use crate::cli::{Args, ProtectedRouteArgs};
use agent_auth::ScopeSet;
use agent_gateway::{
    ProtectedRouteCollection, ProtectedRouteConfig, ProtectedRouteIndex, ProtectedRouteTarget,
};
use std::path::PathBuf;

pub(crate) fn protected_route_index_from_args(
    args: &ProtectedRouteArgs,
) -> anyhow::Result<ProtectedRouteIndex> {
    ProtectedRouteIndex::from_routes(vec![protected_route_config_from_args(args)?])
        .map_err(Into::into)
}

pub(crate) fn protected_route_config_from_args(
    args: &ProtectedRouteArgs,
) -> anyhow::Result<ProtectedRouteConfig> {
    if args.auth_servers.is_empty() {
        anyhow::bail!("--auth-server is required for protected route commands");
    }
    let resource_uri = args
        .resource
        .clone()
        .unwrap_or_else(|| format!("https://{}{}", args.host, args.path));
    Ok(ProtectedRouteConfig {
        name: args.server.clone(),
        enabled: true,
        public_host: args.host.clone(),
        public_path: args.path.clone(),
        resource_uri,
        authorization_servers: args.auth_servers.clone(),
        required_scopes: ScopeSet::parse(&args.scopes)?,
        target: ProtectedRouteTarget::UpstreamMcp {
            server_id: agent_protocol::McpServerId::new(&args.server),
        },
    })
}

pub(crate) fn load_protected_route_collection(
    path: &PathBuf,
) -> anyhow::Result<ProtectedRouteCollection> {
    if !path.exists() {
        return Ok(ProtectedRouteCollection::default());
    }
    let raw = std::fs::read_to_string(path)?;
    let collection: ProtectedRouteCollection = serde_json::from_str(&raw)?;
    collection.validate()?;
    Ok(collection)
}

pub(crate) fn write_protected_route_collection(
    path: &PathBuf,
    routes: &ProtectedRouteCollection,
) -> anyhow::Result<()> {
    routes.validate()?;
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(routes)?)?;
    Ok(())
}

pub(crate) fn protected_route_index(args: &Args) -> anyhow::Result<Option<ProtectedRouteIndex>> {
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
mod tests;
