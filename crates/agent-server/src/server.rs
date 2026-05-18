use crate::{
    backend::GatewayMcpApiBackend,
    cli::Args,
    config::load_mcp_configs_from,
    oauth_store::{oauth_router_for_store_args, parse_oauth_key},
    routes::protected_route_index,
};
use agent_api::{
    GatewayApi, gateway_router, protected_mcp_router, protected_mcp_router_with_oauth_store,
};
use agent_gateway::ProtectedRouteIndex;
use agent_store::SqliteOAuthStore;

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
    routes: ProtectedRouteIndex,
    runtime: std::sync::Arc<agent_runtime::McpRuntime>,
) -> anyhow::Result<axum::Router> {
    match (oauth_store, oauth_key_hex) {
        (Some(path), Some(key_hex)) => {
            let store = SqliteOAuthStore::open(path, parse_oauth_key(key_hex)?)?;
            Ok(protected_mcp_router_with_oauth_store(
                routes, runtime, store,
            ))
        }
        _ => Ok(protected_mcp_router(routes, runtime)),
    }
}
