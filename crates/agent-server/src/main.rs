mod backend;
mod cli;
mod config;
mod oauth_store;
mod routes;
mod server;

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = cli::Args::parse();
    if let Some(command) = args.command {
        return cli::run_command(command).await;
    }

    server::run(args).await
}
