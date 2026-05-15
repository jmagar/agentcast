use agent_api::{
    GatewayApi, GatewayApiSearchResult, gateway_router, oauth_router, oauth_router_with_store,
    protected_mcp_router, protected_mcp_router_with_oauth_store,
};
use agent_auth::{
    OAuthCallback, OAuthClientRegistration, OAuthCredential, OAuthProviderMetadata,
    OAuthRefreshResult, ScopeSet,
};
use agent_config::{
    AgentConfig, discover_known_mcp_configs, load_from_path, merge_env_file, parse_mcp_json,
    write_to_path,
};
use agent_gateway::{
    BeginAuthorization, GatewayOAuthService, ProtectedRouteCollection, ProtectedRouteConfig,
    ProtectedRouteIndex, ProtectedRouteTarget,
};
use agent_mcp::{
    GatewayMcpAction, GatewayMcpBackend, GatewayMcpPrompt, GatewayMcpResource,
    GatewayMcpSearchResult, GatewayMcpServer, GatewayMcpStatus,
};
use agent_protocol::McpServerConfig;
use agent_registry::{McpRegistryClient, McpRegistryResponse, NormalizedMcpServer, search_servers};
use agent_store::{InMemoryOAuthStore, OAuthStore, SqliteOAuthStore};
use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use serde_json::Value;
use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf, sync::Arc};

#[derive(Debug, Parser)]
#[command(name = "agentcast", version, about = "AgentCast gateway server")]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
    #[arg(long, default_value = "127.0.0.1:8787")]
    listen: SocketAddr,
    #[arg(long)]
    mcp_config: Option<PathBuf>,
    #[arg(long)]
    discover_mcp: bool,
    #[arg(long)]
    enable_imported: bool,
    #[arg(long)]
    mcp_stdio: bool,
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
    #[arg(long)]
    oauth_store: Option<PathBuf>,
    #[arg(long, env = "AGENTCAST_OAUTH_KEY_HEX")]
    oauth_key_hex: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Gateway {
        #[command(subcommand)]
        command: Box<GatewayCommand>,
    },
    Registry {
        #[command(subcommand)]
        command: RegistryCommand,
    },
    Marketplace {
        #[command(subcommand)]
        command: MarketplaceCommand,
    },
}

#[derive(Debug, Subcommand)]
enum GatewayCommand {
    Actions(GatewayCommandArgs),
    Search {
        #[command(flatten)]
        config: GatewayCommandArgs,
        q: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    Call {
        #[command(flatten)]
        config: GatewayCommandArgs,
        action_id: String,
        #[arg(long, default_value = "{}")]
        arguments: String,
    },
    ReadResource {
        #[command(flatten)]
        config: GatewayCommandArgs,
        server_id: String,
        uri: String,
    },
    GetPrompt {
        #[command(flatten)]
        config: GatewayCommandArgs,
        server_id: String,
        name: String,
        #[arg(long, default_value = "{}")]
        arguments: String,
    },
    ProtectedRoute {
        #[command(subcommand)]
        command: ProtectedRouteCommand,
    },
    #[command(name = "oauth")]
    OAuth {
        #[command(subcommand)]
        command: OAuthCommand,
    },
}

#[derive(Debug, Clone, ClapArgs)]
struct GatewayCommandArgs {
    #[arg(long)]
    mcp_config: Option<PathBuf>,
    #[arg(long)]
    discover_mcp: bool,
    #[arg(long)]
    enable_imported: bool,
    #[arg(long, value_enum, default_value_t = CliOutputFormat::Json)]
    output: CliOutputFormat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum CliOutputFormat {
    Json,
    Text,
}

#[derive(Debug, Subcommand)]
enum RegistryCommand {
    Search {
        q: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long)]
        registry_response: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum MarketplaceCommand {
    PlanMcp {
        #[arg(long)]
        server_json: String,
    },
    ApplyMcp {
        #[arg(long)]
        server_json: String,
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        write: bool,
        #[arg(long)]
        env_json: Option<String>,
        #[arg(long)]
        env_file: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum ProtectedRouteCommand {
    Show(ProtectedRouteArgs),
    Metadata(ProtectedRouteArgs),
    List {
        #[arg(long)]
        routes_file: PathBuf,
    },
    Add {
        #[arg(long)]
        routes_file: PathBuf,
        #[command(flatten)]
        route: ProtectedRouteArgs,
    },
    Remove {
        #[arg(long)]
        routes_file: PathBuf,
        #[arg(long)]
        name: String,
    },
    Status {
        #[arg(long)]
        routes_file: PathBuf,
        #[arg(long)]
        name: String,
    },
    Test {
        #[arg(long)]
        routes_file: PathBuf,
        #[arg(long)]
        host: String,
        #[arg(long)]
        path: String,
    },
}

#[derive(Debug, Clone, ClapArgs)]
struct ProtectedRouteArgs {
    #[arg(long)]
    host: String,
    #[arg(long)]
    path: String,
    #[arg(long)]
    server: String,
    #[arg(long)]
    resource: Option<String>,
    #[arg(long = "auth-server")]
    auth_servers: Vec<String>,
    #[arg(long, default_value = "mcp:read")]
    scopes: String,
}

#[derive(Debug, Subcommand)]
enum OAuthCommand {
    Probe {
        issuer_url: String,
        #[arg(long)]
        metadata: String,
    },
    Authorize {
        #[command(flatten)]
        store: OAuthStoreArgs,
        #[arg(long)]
        issuer_url: String,
        #[arg(long)]
        metadata: String,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        upstream_id: String,
        #[arg(long)]
        client_id: String,
        #[arg(long)]
        redirect_uri: String,
        #[arg(long)]
        resource_uri: String,
        #[arg(long)]
        state: String,
        #[arg(long)]
        code_challenge: String,
        #[arg(long)]
        expires_at_unix: u64,
        #[arg(long)]
        challenge_scopes: Option<String>,
        #[arg(long)]
        protected_resource_scopes: Option<String>,
    },
    Callback {
        #[command(flatten)]
        store: OAuthStoreArgs,
        #[arg(long)]
        state: String,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        code: String,
        #[arg(long)]
        credential: String,
        #[arg(long)]
        now_unix: u64,
    },
    Status {
        #[command(flatten)]
        store: OAuthStoreArgs,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        upstream_id: String,
        #[arg(long)]
        now_unix: u64,
    },
    Refresh {
        #[command(flatten)]
        store: OAuthStoreArgs,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        upstream_id: String,
        #[arg(long)]
        result: String,
    },
    Clear {
        #[command(flatten)]
        store: OAuthStoreArgs,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        upstream_id: String,
    },
    Register {
        #[command(flatten)]
        store: OAuthStoreArgs,
        #[arg(long)]
        registration: String,
    },
}

#[derive(Debug, Clone, Default, ClapArgs)]
struct OAuthStoreArgs {
    #[arg(long)]
    oauth_store: Option<PathBuf>,
    #[arg(long, env = "AGENTCAST_OAUTH_KEY_HEX")]
    oauth_key_hex: Option<String>,
}

#[derive(Debug, Serialize)]
struct OAuthStatusOutput {
    subject: String,
    upstream_id: String,
    status: agent_auth::OAuthStatus,
}

#[derive(Debug, Serialize)]
struct OAuthAuthorizeOutput {
    authorization_url: String,
    selected_scopes: Vec<String>,
}

#[derive(Debug, Serialize)]
struct OAuthClientRegistrationOutput {
    subject: String,
    upstream_id: String,
    client_id: String,
    has_client_secret: bool,
    client_id_issued_at_unix: Option<u64>,
    client_secret_expires_at_unix: Option<u64>,
}

impl From<OAuthClientRegistration> for OAuthClientRegistrationOutput {
    fn from(registration: OAuthClientRegistration) -> Self {
        Self {
            subject: registration.subject,
            upstream_id: registration.upstream_id,
            client_id: registration.client_id,
            has_client_secret: registration.client_secret.is_some(),
            client_id_issued_at_unix: registration.client_id_issued_at_unix,
            client_secret_expires_at_unix: registration.client_secret_expires_at_unix,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    if let Some(command) = args.command {
        return run_command(command).await;
    }
    let configs = load_mcp_configs(&args)?;
    let protected_routes = protected_route_index(&args)?;
    let api = GatewayApi::start(configs).await;
    if args.mcp_stdio {
        agent_mcp::serve_gateway_stdio(GatewayMcpApiBackend::new(api))
            .await
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        return Ok(());
    }
    let runtime = api.runtime();
    let mut router = gateway_router(api).merge(oauth_router_for_args(&args)?);
    if let Some(routes) = protected_routes {
        router = router.merge(protected_mcp_router_for_args(&args, routes, runtime)?);
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

async fn run_command(command: Command) -> anyhow::Result<()> {
    match command {
        Command::Gateway { command } => run_gateway_command(*command).await,
        Command::Registry { command } => run_registry_command(command).await,
        Command::Marketplace { command } => run_marketplace_command(command),
    }
}

async fn run_gateway_command(command: GatewayCommand) -> anyhow::Result<()> {
    match command {
        GatewayCommand::Actions(config) => {
            let handlers = gateway_handlers(&config).await?;
            let rows = handlers.list_actions();
            match config.output {
                CliOutputFormat::Json => print_json(&rows),
                CliOutputFormat::Text => {
                    print_text(agent_cli::GatewayCliView::render_actions_table(&rows))
                }
            }
        }
        GatewayCommand::Search { config, q, limit } => {
            let handlers = gateway_handlers(&config).await?;
            let rows = handlers.search_actions(&q, limit);
            match config.output {
                CliOutputFormat::Json => print_json(&rows),
                CliOutputFormat::Text => {
                    print_text(agent_cli::GatewayCliView::render_search_table(&rows))
                }
            }
        }
        GatewayCommand::Call {
            config,
            action_id,
            arguments,
        } => {
            let handlers = gateway_handlers(&config).await?;
            let arguments = parse_json_argument(&arguments)?;
            let output = handlers.call_action(&action_id, arguments).await?;
            print_json(&output)
        }
        GatewayCommand::ReadResource {
            config,
            server_id,
            uri,
        } => {
            let handlers = gateway_handlers(&config).await?;
            let output = handlers.read_resource(&server_id, &uri).await?;
            print_json(&output)
        }
        GatewayCommand::GetPrompt {
            config,
            server_id,
            name,
            arguments,
        } => {
            let handlers = gateway_handlers(&config).await?;
            let arguments = parse_json_object_argument(&arguments)?;
            let output = handlers
                .get_prompt(&server_id, &name, Some(arguments))
                .await?;
            print_json(&output)
        }
        GatewayCommand::ProtectedRoute { command } => run_protected_route_command(command),
        GatewayCommand::OAuth { command } => run_oauth_command(command),
    }
}

fn run_protected_route_command(command: ProtectedRouteCommand) -> anyhow::Result<()> {
    match command {
        ProtectedRouteCommand::Show(args) => {
            let index = protected_route_index_from_args(&args)?;
            let route = index
                .resolve(&args.host, &args.path)
                .ok_or_else(|| anyhow::anyhow!("protected route did not resolve"))?;
            print_json(&agent_cli::GatewayCliView::protected_route(route))
        }
        ProtectedRouteCommand::Metadata(args) => {
            let index = protected_route_index_from_args(&args)?;
            let route = index
                .resolve(&args.host, &args.path)
                .ok_or_else(|| anyhow::anyhow!("protected route did not resolve"))?;
            print_json(&route.protected_resource_metadata())
        }
        ProtectedRouteCommand::List { routes_file } => {
            let routes = load_protected_route_collection(&routes_file)?;
            print_json(&routes.list().to_vec())
        }
        ProtectedRouteCommand::Add { routes_file, route } => {
            let mut routes = load_protected_route_collection(&routes_file)?;
            let config = protected_route_config_from_args(&route)?;
            routes.upsert(config.clone())?;
            write_protected_route_collection(&routes_file, &routes)?;
            print_json(&config)
        }
        ProtectedRouteCommand::Remove { routes_file, name } => {
            let mut routes = load_protected_route_collection(&routes_file)?;
            let removed = routes.remove(&name)?;
            write_protected_route_collection(&routes_file, &routes)?;
            print_json(&removed)
        }
        ProtectedRouteCommand::Status { routes_file, name } => {
            let routes = load_protected_route_collection(&routes_file)?;
            print_json(&routes.status(&name)?)
        }
        ProtectedRouteCommand::Test {
            routes_file,
            host,
            path,
        } => {
            let routes = load_protected_route_collection(&routes_file)?;
            let resolved = routes.test(&host, &path)?;
            print_json(&serde_json::json!({
                "matched": resolved.is_some(),
                "route_name": resolved.map(|route| route.name),
            }))
        }
    }
}

async fn run_registry_command(command: RegistryCommand) -> anyhow::Result<()> {
    match command {
        RegistryCommand::Search {
            q,
            limit,
            registry_response,
        } => {
            let servers = if let Some(raw) = registry_response {
                parse_json::<McpRegistryResponse>(&raw)?.normalize()?
            } else {
                McpRegistryClient::official()?
                    .list_servers(Some(&q), Some(limit), None)
                    .await?
            };
            print_json(&search_servers(&servers, &q, limit))
        }
    }
}

fn run_marketplace_command(command: MarketplaceCommand) -> anyhow::Result<()> {
    match command {
        MarketplaceCommand::PlanMcp { server_json } => {
            let server = parse_json::<NormalizedMcpServer>(&server_json)?;
            let plan = agent_marketplace::plan_mcp_server_install(&server)?;
            print_json(&plan)
        }
        MarketplaceCommand::ApplyMcp {
            server_json,
            config,
            write,
            env_json,
            env_file,
        } => {
            let server = parse_json::<NormalizedMcpServer>(&server_json)?;
            let plan = agent_marketplace::plan_mcp_server_install(&server)?;
            let env_values = env_json
                .as_deref()
                .map(parse_json::<BTreeMap<String, String>>)
                .transpose()?
                .unwrap_or_default();
            let env_resolution = agent_marketplace::resolve_install_env(&server, &env_values)?;
            let env_merge = agent_marketplace::install_env_merge(&env_resolution)?;
            let mut agent_config = if config.exists() {
                load_from_path(&config)?
            } else {
                AgentConfig::default()
            };
            let result = agent_marketplace::apply_install_plan_to_config(&mut agent_config, &plan)?;
            if write {
                write_to_path(&config, &agent_config)?;
                if let Some(env_file) = &env_file {
                    merge_env_file(env_file, &env_merge)?;
                }
            }
            print_json(&serde_json::json!({
                "written": write,
                "result": result,
                "config": agent_config,
                "env_values": env_resolution.values,
            }))
        }
    }
}

fn run_oauth_command(command: OAuthCommand) -> anyhow::Result<()> {
    match command {
        OAuthCommand::Probe {
            issuer_url,
            metadata,
        } => {
            let metadata = parse_json::<OAuthProviderMetadata>(&metadata)?;
            let service = oauth_service(OAuthStoreArgs::default())?;
            print_json(&service.probe_metadata(&issuer_url, &metadata)?)
        }
        OAuthCommand::Authorize {
            store,
            issuer_url,
            metadata,
            subject,
            upstream_id,
            client_id,
            redirect_uri,
            resource_uri,
            state,
            code_challenge,
            expires_at_unix,
            challenge_scopes,
            protected_resource_scopes,
        } => {
            let metadata = parse_json::<OAuthProviderMetadata>(&metadata)?;
            let mut service = oauth_service(store)?;
            let result = service.begin_authorization(
                BeginAuthorization {
                    issuer_url,
                    subject,
                    upstream_id,
                    client_id,
                    redirect_uri,
                    resource_uri,
                    state,
                    code_challenge,
                    expires_at_unix,
                    challenge_scopes: parse_optional_scopes(challenge_scopes)?,
                    protected_resource_scopes: parse_optional_scopes(protected_resource_scopes)?,
                },
                &metadata,
            )?;
            print_json(&OAuthAuthorizeOutput {
                authorization_url: result.authorization_url,
                selected_scopes: result
                    .selected_scope
                    .map(|scope| scope.as_slice().to_vec())
                    .unwrap_or_default(),
            })
        }
        OAuthCommand::Callback {
            store,
            state,
            subject,
            code,
            credential,
            now_unix,
        } => {
            let credential = parse_json::<OAuthCredential>(&credential)?;
            let mut service = oauth_service(store)?;
            service.complete_callback(
                OAuthCallback {
                    state,
                    subject,
                    code,
                },
                credential,
                now_unix,
            )?;
            print_json(&serde_json::json!({ "status": "connected" }))
        }
        OAuthCommand::Status {
            store,
            subject,
            upstream_id,
            now_unix,
        } => {
            let service = oauth_service(store)?;
            let status = service.status(&subject, &upstream_id, now_unix)?;
            print_json(&OAuthStatusOutput {
                subject,
                upstream_id,
                status,
            })
        }
        OAuthCommand::Refresh {
            store,
            subject,
            upstream_id,
            result,
        } => {
            let result = parse_json::<OAuthRefreshResult>(&result)?;
            let mut service = oauth_service(store)?;
            let status = service.finish_refresh_success(&subject, &upstream_id, result)?;
            print_json(&OAuthStatusOutput {
                subject,
                upstream_id,
                status,
            })
        }
        OAuthCommand::Clear {
            store,
            subject,
            upstream_id,
        } => {
            let mut service = oauth_service(store)?;
            service.clear(&subject, &upstream_id)?;
            print_json(&serde_json::json!({ "cleared": true }))
        }
        OAuthCommand::Register {
            store,
            registration,
        } => {
            let registration = parse_json::<OAuthClientRegistration>(&registration)?;
            let mut service = oauth_service(store)?;
            service.put_client_registration(registration.clone())?;
            print_json(&OAuthClientRegistrationOutput::from(registration))
        }
    }
}

async fn gateway_handlers(
    config: &GatewayCommandArgs,
) -> anyhow::Result<agent_cli::GatewayCliHandlers> {
    Ok(agent_cli::GatewayCliHandlers::start(load_mcp_configs_from(
        config.mcp_config.as_ref(),
        config.discover_mcp,
        config.enable_imported,
    )?)
    .await)
}

fn print_json(value: &impl serde::Serialize) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn print_text(value: String) -> anyhow::Result<()> {
    println!("{value}");
    Ok(())
}

fn parse_json_argument(raw: &str) -> anyhow::Result<Value> {
    Ok(serde_json::from_str(raw)?)
}

fn parse_json<T>(raw: &str) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    Ok(serde_json::from_str(raw)?)
}

fn parse_json_object_argument(raw: &str) -> anyhow::Result<serde_json::Map<String, Value>> {
    match parse_json_argument(raw)? {
        Value::Object(object) => Ok(object),
        _ => anyhow::bail!("--arguments must be a JSON object"),
    }
}

fn parse_optional_scopes(raw: Option<String>) -> anyhow::Result<Option<ScopeSet>> {
    raw.as_deref()
        .map(ScopeSet::parse)
        .transpose()
        .map_err(Into::into)
}

fn protected_route_index_from_args(
    args: &ProtectedRouteArgs,
) -> anyhow::Result<ProtectedRouteIndex> {
    ProtectedRouteIndex::from_routes(vec![protected_route_config_from_args(args)?])
        .map_err(Into::into)
}

fn protected_route_config_from_args(
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

fn load_protected_route_collection(path: &PathBuf) -> anyhow::Result<ProtectedRouteCollection> {
    if !path.exists() {
        return Ok(ProtectedRouteCollection::default());
    }
    let raw = std::fs::read_to_string(path)?;
    let collection: ProtectedRouteCollection = serde_json::from_str(&raw)?;
    collection.validate()?;
    Ok(collection)
}

fn write_protected_route_collection(
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

fn oauth_service(
    args: OAuthStoreArgs,
) -> anyhow::Result<GatewayOAuthService<Box<dyn OAuthStore + Send>>> {
    match (args.oauth_store, args.oauth_key_hex) {
        (None, None) => Ok(GatewayOAuthService::new(
            Box::new(InMemoryOAuthStore::default()) as Box<dyn OAuthStore + Send>,
        )),
        (Some(path), Some(key_hex)) => Ok(GatewayOAuthService::new(
            Box::new(SqliteOAuthStore::open(path, parse_oauth_key(&key_hex)?)?)
                as Box<dyn OAuthStore + Send>,
        )),
        (Some(_), None) => anyhow::bail!("--oauth-key-hex is required with --oauth-store"),
        (None, Some(_)) => anyhow::bail!("--oauth-store is required with --oauth-key-hex"),
    }
}

struct GatewayMcpApiBackend {
    api: Arc<GatewayApi>,
}

impl GatewayMcpApiBackend {
    fn new(api: GatewayApi) -> Self {
        Self { api: Arc::new(api) }
    }
}

#[async_trait::async_trait]
impl GatewayMcpBackend for GatewayMcpApiBackend {
    fn gateway_status(&self) -> GatewayMcpStatus {
        let snapshots = self.api.runtime().snapshots();
        GatewayMcpStatus {
            server_count: snapshots.len(),
            action_count: self.api.list_actions().len(),
        }
    }

    fn list_servers(&self) -> Vec<GatewayMcpServer> {
        self.api
            .runtime()
            .snapshots()
            .into_iter()
            .map(|snapshot| GatewayMcpServer {
                id: snapshot.server_id.to_string(),
                name: snapshot.server_name,
                status: serde_json::to_value(snapshot.status)
                    .ok()
                    .and_then(|value| value.as_str().map(ToOwned::to_owned))
                    .unwrap_or_else(|| "unknown".to_string()),
                tool_count: snapshot.tools.len(),
                resource_count: snapshot.resources.len(),
                prompt_count: snapshot.prompts.len(),
            })
            .collect()
    }

    fn list_actions(&self) -> Vec<GatewayMcpAction> {
        self.api
            .list_actions()
            .into_iter()
            .map(|action| GatewayMcpAction {
                id: action.id,
                name: action.display_name,
                description: action.description,
            })
            .collect()
    }

    fn search_actions(&self, query: &str, limit: usize) -> Vec<GatewayMcpSearchResult> {
        self.api
            .search_actions(query, limit)
            .into_iter()
            .map(search_result)
            .collect()
    }

    async fn call_action(&self, action_id: &str, arguments: Value) -> Result<Value, String> {
        self.api
            .call_action(action_id, arguments)
            .await
            .map_err(|error| error.to_string())
    }

    fn list_resources(&self, server_id: Option<&str>) -> Vec<GatewayMcpResource> {
        self.api
            .runtime()
            .snapshots()
            .into_iter()
            .filter(|snapshot| server_id.is_none_or(|id| snapshot.server_id.as_str() == id))
            .flat_map(|snapshot| {
                let server_id = snapshot.server_id.to_string();
                snapshot
                    .resources
                    .into_iter()
                    .map(move |resource| GatewayMcpResource {
                        server_id: server_id.clone(),
                        uri: resource.uri,
                        name: resource.name,
                        title: resource.title,
                        description: resource.description,
                        mime_type: resource.mime_type,
                    })
            })
            .collect()
    }

    async fn read_resource(&self, server_id: &str, uri: &str) -> Result<Value, String> {
        self.api
            .read_resource(server_id, uri)
            .await
            .map_err(|error| error.to_string())
    }

    fn list_prompts(&self, server_id: Option<&str>) -> Vec<GatewayMcpPrompt> {
        self.api
            .runtime()
            .snapshots()
            .into_iter()
            .filter(|snapshot| server_id.is_none_or(|id| snapshot.server_id.as_str() == id))
            .flat_map(|snapshot| {
                let server_id = snapshot.server_id.to_string();
                snapshot
                    .prompts
                    .into_iter()
                    .map(move |prompt| GatewayMcpPrompt {
                        server_id: server_id.clone(),
                        name: prompt.name,
                        title: prompt.title,
                        description: prompt.description,
                        arguments: prompt.arguments,
                    })
            })
            .collect()
    }

    async fn get_prompt(
        &self,
        server_id: &str,
        name: &str,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<Value, String> {
        self.api
            .runtime()
            .get_prompt(
                &agent_protocol::McpServerId::new(server_id),
                name,
                arguments,
            )
            .await
            .map_err(|error| error.to_string())
    }
}

fn search_result(result: GatewayApiSearchResult) -> GatewayMcpSearchResult {
    GatewayMcpSearchResult {
        action_id: result.action_id,
        name: result.name,
        score: result.score,
        match_kind: "Ranked".to_string(),
        truncated: false,
    }
}

fn protected_mcp_router_for_args(
    args: &Args,
    routes: ProtectedRouteIndex,
    runtime: std::sync::Arc<agent_runtime::McpRuntime>,
) -> anyhow::Result<axum::Router> {
    match (&args.oauth_store, &args.oauth_key_hex) {
        (Some(path), Some(key_hex)) => {
            let store = SqliteOAuthStore::open(path, parse_oauth_key(key_hex)?)?;
            Ok(protected_mcp_router_with_oauth_store(
                routes, runtime, store,
            ))
        }
        _ => Ok(protected_mcp_router(routes, runtime)),
    }
}

fn oauth_router_for_args(args: &Args) -> anyhow::Result<axum::Router> {
    match (&args.oauth_store, &args.oauth_key_hex) {
        (None, None) => Ok(oauth_router()),
        (Some(path), Some(key_hex)) => {
            let store = SqliteOAuthStore::open(path, parse_oauth_key(key_hex)?)?;
            Ok(oauth_router_with_store(store))
        }
        (Some(_), None) => anyhow::bail!("--oauth-key-hex is required with --oauth-store"),
        (None, Some(_)) => anyhow::bail!("--oauth-store is required with --oauth-key-hex"),
    }
}

fn parse_oauth_key(raw: &str) -> anyhow::Result<[u8; 32]> {
    let bytes = hex::decode(raw.trim())?;
    if bytes.len() != 32 {
        anyhow::bail!("OAuth store key must decode to 32 bytes");
    }
    let mut key = [0_u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

fn load_mcp_configs(args: &Args) -> anyhow::Result<Vec<McpServerConfig>> {
    load_mcp_configs_from(
        args.mcp_config.as_ref(),
        args.discover_mcp,
        args.enable_imported,
    )
}

fn load_mcp_configs_from(
    mcp_config: Option<&PathBuf>,
    discover_mcp: bool,
    enable_imported: bool,
) -> anyhow::Result<Vec<McpServerConfig>> {
    let mut configs = Vec::new();
    if let Some(path) = mcp_config {
        let raw = std::fs::read_to_string(path)?;
        configs.extend(parse_mcp_json(&raw)?);
    }

    if discover_mcp {
        let home = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        configs.extend(
            discover_known_mcp_configs(&home)
                .into_iter()
                .map(|discovered| discovered.config),
        );
    }

    configs = dedupe_configs(configs);
    if enable_imported {
        for config in &mut configs {
            config.enabled = true;
        }
    }
    Ok(configs)
}

fn dedupe_configs(configs: Vec<McpServerConfig>) -> Vec<McpServerConfig> {
    let mut seen = std::collections::BTreeSet::new();
    configs
        .into_iter()
        .filter(|config| seen.insert(config.id.clone()))
        .collect()
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
        let configs = load_mcp_configs(&args_with_config(path, false)).expect("configs");

        assert_eq!(configs.len(), 1);
        assert!(!configs[0].enabled);
    }

    #[test]
    fn load_mcp_configs_can_enable_operator_supplied_imports() {
        let path = write_config();
        let configs = load_mcp_configs(&args_with_config(path, true)).expect("configs");

        assert_eq!(configs.len(), 1);
        assert!(configs[0].enabled);
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

    #[test]
    fn empty_config_without_path_is_valid() {
        let configs = load_mcp_configs(&args_without_config()).expect("configs");
        assert!(configs.is_empty());
    }

    #[test]
    fn server_config_shape_stays_in_protocol_models() {
        let configs = load_mcp_configs(&args_without_config()).expect("configs");
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
            command: None,
            listen: "127.0.0.1:8787".parse().expect("listen"),
            mcp_config: None,
            discover_mcp: false,
            enable_imported: false,
            mcp_stdio: false,
            protected_mcp_host: Some("mcp.example.test".to_string()),
            protected_mcp_path: None,
            protected_mcp_server: Some("fixture".to_string()),
            protected_mcp_resource: None,
            protected_mcp_auth_servers: vec!["https://auth.example.test".to_string()],
            protected_mcp_scopes: "mcp:read".to_string(),
            oauth_store: None,
            oauth_key_hex: None,
        };

        assert!(protected_route_index(&args).is_err());
    }

    #[test]
    fn protected_route_index_builds_generic_upstream_route() {
        let args = Args {
            command: None,
            listen: "127.0.0.1:8787".parse().expect("listen"),
            mcp_config: None,
            discover_mcp: false,
            enable_imported: false,
            mcp_stdio: false,
            protected_mcp_host: Some("mcp.example.test".to_string()),
            protected_mcp_path: Some("/syslog".to_string()),
            protected_mcp_server: Some("fixture".to_string()),
            protected_mcp_resource: Some("https://mcp.example.test/syslog".to_string()),
            protected_mcp_auth_servers: vec!["https://auth.example.test".to_string()],
            protected_mcp_scopes: "mcp:read".to_string(),
            oauth_store: None,
            oauth_key_hex: None,
        };

        let routes = protected_route_index(&args)
            .expect("route index")
            .expect("protected routes");
        assert!(routes.resolve("mcp.example.test", "/syslog").is_some());
    }

    fn args_with_config(path: PathBuf, enable_imported: bool) -> Args {
        Args {
            command: None,
            listen: "127.0.0.1:8787".parse().expect("listen"),
            mcp_config: Some(path),
            discover_mcp: false,
            enable_imported,
            mcp_stdio: false,
            protected_mcp_host: None,
            protected_mcp_path: None,
            protected_mcp_server: None,
            protected_mcp_resource: None,
            protected_mcp_auth_servers: Vec::new(),
            protected_mcp_scopes: "mcp:read".to_string(),
            oauth_store: None,
            oauth_key_hex: None,
        }
    }

    fn args_without_config() -> Args {
        Args {
            command: None,
            listen: "127.0.0.1:8787".parse().expect("listen"),
            mcp_config: None,
            discover_mcp: false,
            enable_imported: false,
            mcp_stdio: false,
            protected_mcp_host: None,
            protected_mcp_path: None,
            protected_mcp_server: None,
            protected_mcp_resource: None,
            protected_mcp_auth_servers: Vec::new(),
            protected_mcp_scopes: "mcp:read".to_string(),
            oauth_store: None,
            oauth_key_hex: None,
        }
    }

    #[test]
    fn oauth_key_requires_32_decoded_bytes() {
        assert!(parse_oauth_key(&"07".repeat(32)).is_ok());
        assert!(parse_oauth_key("07").is_err());
    }

    #[test]
    fn oauth_store_flags_must_be_complete() {
        let mut args = args_without_config();
        args.oauth_store = Some(PathBuf::from("oauth.db"));
        assert!(oauth_router_for_args(&args).is_err());

        let mut args = args_without_config();
        args.oauth_key_hex = Some("07".repeat(32));
        assert!(oauth_router_for_args(&args).is_err());
    }

    #[test]
    fn parses_gateway_cli_subcommands() {
        let args = Args::try_parse_from([
            "agentcast",
            "gateway",
            "call",
            "--mcp-config",
            "mcp.json",
            "--enable-imported",
            "mcp:fixture:echo",
            "--arguments",
            r#"{"message":"hi"}"#,
        ])
        .expect("parse");

        let Some(Command::Gateway { command }) = args.command else {
            panic!("gateway call command");
        };
        let GatewayCommand::Call {
            config,
            action_id,
            arguments,
        } = *command
        else {
            panic!("gateway call command");
        };

        assert_eq!(config.mcp_config, Some(PathBuf::from("mcp.json")));
        assert!(config.enable_imported);
        assert_eq!(action_id, "mcp:fixture:echo");
        assert_eq!(arguments, r#"{"message":"hi"}"#);
    }

    #[test]
    fn parses_protected_route_cli_subcommands() {
        let args = Args::try_parse_from([
            "agentcast",
            "gateway",
            "protected-route",
            "metadata",
            "--host",
            "mcp.example.test",
            "--path",
            "/syslog",
            "--server",
            "syslog",
            "--auth-server",
            "https://auth.example.test",
        ])
        .expect("parse");

        let Some(Command::Gateway { command }) = args.command else {
            panic!("protected route metadata command");
        };
        let GatewayCommand::ProtectedRoute {
            command: ProtectedRouteCommand::Metadata(route),
        } = *command
        else {
            panic!("protected route metadata command");
        };

        assert_eq!(route.host, "mcp.example.test");
        assert_eq!(route.path, "/syslog");
        assert_eq!(route.server, "syslog");
        assert_eq!(route.auth_servers, vec!["https://auth.example.test"]);
    }

    #[test]
    fn parses_oauth_cli_subcommands_without_exposing_secret_result() {
        let args = Args::try_parse_from([
            "agentcast",
            "gateway",
            "oauth",
            "register",
            "--registration",
            r#"{"subject":"user-1","upstream_id":"github","client_id":"client-1","client_secret":"secret","client_id_issued_at_unix":1,"client_secret_expires_at_unix":2}"#,
        ])
        .expect("parse");

        let Some(Command::Gateway { command }) = args.command else {
            panic!("oauth register command");
        };
        let GatewayCommand::OAuth {
            command: OAuthCommand::Register { registration, .. },
        } = *command
        else {
            panic!("oauth register command");
        };

        let registration = parse_json::<OAuthClientRegistration>(&registration).expect("json");
        let output = OAuthClientRegistrationOutput::from(registration);
        assert_eq!(output.client_id, "client-1");
        assert!(output.has_client_secret);
    }

    #[test]
    fn parses_registry_search_with_offline_response() {
        let args = Args::try_parse_from([
            "agentcast",
            "registry",
            "search",
            "filesystem",
            "--limit",
            "5",
            "--registry-response",
            r#"{"servers":[]}"#,
        ])
        .expect("parse");

        let Some(Command::Registry {
            command:
                RegistryCommand::Search {
                    q,
                    limit,
                    registry_response,
                },
        }) = args.command
        else {
            panic!("registry search command");
        };

        assert_eq!(q, "filesystem");
        assert_eq!(limit, 5);
        assert_eq!(registry_response.as_deref(), Some(r#"{"servers":[]}"#));
    }

    #[test]
    fn parses_marketplace_plan_and_apply_commands() {
        let server_json = normalized_server_json();
        let args = Args::try_parse_from([
            "agentcast",
            "marketplace",
            "plan-mcp",
            "--server-json",
            &server_json,
        ])
        .expect("parse");

        assert!(matches!(
            args.command,
            Some(Command::Marketplace {
                command: MarketplaceCommand::PlanMcp { .. }
            })
        ));

        let args = Args::try_parse_from([
            "agentcast",
            "marketplace",
            "apply-mcp",
            "--server-json",
            &server_json,
            "--config",
            "agentcast.toml",
            "--write",
        ])
        .expect("parse");

        let Some(Command::Marketplace {
            command: MarketplaceCommand::ApplyMcp { config, write, .. },
        }) = args.command
        else {
            panic!("marketplace apply command");
        };
        assert_eq!(config, PathBuf::from("agentcast.toml"));
        assert!(write);
    }

    fn normalized_server_json() -> String {
        serde_json::json!({
            "name": "io.modelcontextprotocol/filesystem",
            "title": null,
            "description": "Filesystem MCP server",
            "latest_version": "0.6.2",
            "packages": [{
                "registry_type": "npm",
                "identifier": "@modelcontextprotocol/server-filesystem",
                "version": "0.6.2",
                "runtime_hint": "npx",
                "transport": "stdio",
                "runtime_arguments": ["-y"],
                "package_arguments": ["/tmp"],
                "environment_variables": []
            }],
            "remotes": [],
            "repository_url": null,
            "website_url": null,
            "provenance": {
                "source": "mcp_registry",
                "source_url": "https://registry.modelcontextprotocol.io"
            },
            "registry_metadata": {}
        })
        .to_string()
    }
}
