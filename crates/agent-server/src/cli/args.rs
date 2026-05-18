use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};
use std::{net::SocketAddr, path::PathBuf};

#[derive(Debug, Parser)]
#[command(name = "agentcast", version, about = "AgentCast gateway server")]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
    #[arg(long, default_value = "127.0.0.1:8787")]
    pub(crate) listen: SocketAddr,
    #[arg(long)]
    pub(crate) mcp_config: Option<PathBuf>,
    #[arg(long)]
    pub(crate) discover_mcp: bool,
    #[arg(long)]
    pub(crate) enable_imported: bool,
    #[arg(long)]
    pub(crate) mcp_stdio: bool,
    #[arg(long)]
    pub(crate) protected_mcp_host: Option<String>,
    #[arg(long)]
    pub(crate) protected_mcp_path: Option<String>,
    #[arg(long)]
    pub(crate) protected_mcp_server: Option<String>,
    #[arg(long)]
    pub(crate) protected_mcp_resource: Option<String>,
    #[arg(long = "protected-mcp-auth-server")]
    pub(crate) protected_mcp_auth_servers: Vec<String>,
    #[arg(long, default_value = "mcp:read")]
    pub(crate) protected_mcp_scopes: String,
    #[arg(long)]
    pub(crate) oauth_store: Option<PathBuf>,
    #[arg(long, env = "AGENTCAST_OAUTH_KEY_HEX")]
    pub(crate) oauth_key_hex: Option<String>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
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
pub(crate) enum GatewayCommand {
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
pub(crate) struct GatewayCommandArgs {
    #[arg(long)]
    pub(crate) mcp_config: Option<PathBuf>,
    #[arg(long)]
    pub(crate) discover_mcp: bool,
    #[arg(long)]
    pub(crate) enable_imported: bool,
    #[arg(long, value_enum, default_value_t = CliOutputFormat::Json)]
    pub(crate) output: CliOutputFormat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum CliOutputFormat {
    Json,
    Text,
}

#[derive(Debug, Subcommand)]
pub(crate) enum RegistryCommand {
    Search {
        q: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long)]
        registry_response: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum MarketplaceCommand {
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
pub(crate) enum ProtectedRouteCommand {
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
pub(crate) struct ProtectedRouteArgs {
    #[arg(long)]
    pub(crate) host: String,
    #[arg(long)]
    pub(crate) path: String,
    #[arg(long)]
    pub(crate) server: String,
    #[arg(long)]
    pub(crate) resource: Option<String>,
    #[arg(long = "auth-server")]
    pub(crate) auth_servers: Vec<String>,
    #[arg(long, default_value = "mcp:read")]
    pub(crate) scopes: String,
}

#[derive(Debug, Subcommand)]
pub(crate) enum OAuthCommand {
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
pub(crate) struct OAuthStoreArgs {
    #[arg(long)]
    pub(crate) oauth_store: Option<PathBuf>,
    #[arg(long, env = "AGENTCAST_OAUTH_KEY_HEX")]
    pub(crate) oauth_key_hex: Option<String>,
}
