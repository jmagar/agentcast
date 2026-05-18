use super::{
    CliOutputFormat, Command, GatewayCommand, GatewayCommandArgs, MarketplaceCommand, OAuthCommand,
    OAuthStoreArgs, ProtectedRouteCommand, RegistryCommand,
};
use crate::{
    config::load_mcp_configs_from,
    oauth_store::oauth_service,
    routes::{
        load_protected_route_collection, protected_route_config_from_args,
        protected_route_index_from_args, write_protected_route_collection,
    },
};
use agent_auth::{
    OAuthCallback, OAuthClientRegistration, OAuthCredential, OAuthProviderMetadata,
    OAuthRefreshResult, ScopeSet,
};
use agent_config::{AgentConfig, load_from_path, merge_env_file, write_to_path};
use agent_gateway::BeginAuthorization;
use agent_registry::{McpRegistryClient, McpRegistryResponse, NormalizedMcpServer, search_servers};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

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
pub(crate) struct OAuthClientRegistrationOutput {
    pub(crate) subject: String,
    pub(crate) upstream_id: String,
    pub(crate) client_id: String,
    pub(crate) has_client_secret: bool,
    pub(crate) client_id_issued_at_unix: Option<u64>,
    pub(crate) client_secret_expires_at_unix: Option<u64>,
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

pub(crate) async fn run_command(command: Command) -> anyhow::Result<()> {
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

pub(crate) fn parse_json<T>(raw: &str) -> anyhow::Result<T>
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
