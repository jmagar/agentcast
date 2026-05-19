use crate::{
    backend::GatewayMcpApiBackend,
    cli::Args,
    config::load_mcp_configs_from,
    oauth_store::{oauth_router_for_store_args, parse_oauth_key},
    routes::protected_route_index,
};
use agent_api::{
    GatewayApi, gateway_router, protected_mcp_router_with_oauth_store_and_verifier,
    protected_mcp_router_with_verifier,
};
use agent_auth::{BearerClaims, StaticBearerTokenVerifier};
use agent_gateway::ProtectedRouteIndex;
use agent_store::SqliteOAuthStore;
use std::sync::Arc;

#[cfg(test)]
mod tests;

pub(crate) async fn run(args: Args) -> anyhow::Result<()> {
    let configs = load_mcp_configs_from(
        args.mcp_config.as_ref(),
        args.discover_mcp,
        args.enable_imported,
    )?;
    let protected_routes = protected_route_index(&args)?;
    let api = GatewayApi::start(configs).await;
    if args.mcp_stdio {
        agent_mcp::serve_gateway_stdio(GatewayMcpApiBackend::new(api))
            .await
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        return Ok(());
    }
    let runtime = api.runtime();
    let mut router = gateway_router(api).merge(oauth_router_for_store_args(
        args.oauth_store.as_ref(),
        args.oauth_key_hex.as_ref(),
    )?);
    if let Some(routes) = protected_routes {
        router = router.merge(protected_mcp_router_for_store_args(
            args.oauth_store.as_ref(),
            args.oauth_key_hex.as_ref(),
            args.protected_mcp_bearer_token.as_ref(),
            routes,
            runtime,
        )?);
    }
    let listener = tokio::net::TcpListener::bind(args.listen).await?;

    tracing::info!(listen = %args.listen, "serving AgentCast gateway API");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::warn!(%error, "failed to install ctrl-c shutdown handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::warn!(%error, "failed to install terminate shutdown handler");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}

fn protected_mcp_router_for_store_args(
    oauth_store: Option<&std::path::PathBuf>,
    oauth_key_hex: Option<&String>,
    bearer_token: Option<&String>,
    routes: ProtectedRouteIndex,
    runtime: Arc<agent_runtime::McpRuntime>,
) -> anyhow::Result<axum::Router> {
    let verifier = protected_mcp_static_verifier(bearer_token, &routes)?;
    match (oauth_store, oauth_key_hex) {
        (Some(path), Some(key_hex)) => {
            let store = SqliteOAuthStore::open(path, parse_oauth_key(key_hex)?)?;
            Ok(protected_mcp_router_with_oauth_store_and_verifier(
                routes, runtime, store, verifier,
            ))
        }
        _ => Ok(protected_mcp_router_with_verifier(
            routes, runtime, verifier,
        )),
    }
}

fn protected_mcp_static_verifier(
    bearer_token: Option<&String>,
    routes: &ProtectedRouteIndex,
) -> anyhow::Result<Arc<StaticBearerTokenVerifier>> {
    let token = bearer_token.ok_or_else(|| {
        anyhow::anyhow!(
            "--protected-mcp-bearer-token or AGENTCAST_PROTECTED_MCP_BEARER_TOKEN is required for protected MCP routes"
        )
    })?;
    let claims = routes
        .routes()
        .next()
        .map(|route| BearerClaims {
            subject: "protected-mcp-static".to_string(),
            audience: route.resource_uri.to_string(),
            scopes: route.required_scopes.clone(),
        })
        .ok_or_else(|| anyhow::anyhow!("at least one protected MCP route is required"))?;
    Ok(Arc::new(StaticBearerTokenVerifier::new([(
        token.clone(),
        claims,
    )])))
}
