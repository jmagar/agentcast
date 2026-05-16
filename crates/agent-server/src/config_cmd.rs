use std::path::PathBuf;

use agent_config::{
    AgentPaths, config_get, config_list, config_paths, config_set, config_unset, config_validate,
    default_paths, env_get, env_list, env_set, env_unset,
};
use clap::Subcommand;
use serde::Serialize;

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    Get {
        #[arg(long)]
        config: Option<PathBuf>,
        key: String,
    },
    Set {
        #[arg(long)]
        config: Option<PathBuf>,
        key: String,
        value: String,
    },
    Unset {
        #[arg(long)]
        config: Option<PathBuf>,
        key: String,
    },
    List {
        #[arg(long)]
        config: Option<PathBuf>,
    },
    Validate {
        #[arg(long)]
        config: Option<PathBuf>,
    },
    Path {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        env_file: Option<PathBuf>,
    },
    Env {
        #[command(subcommand)]
        command: ConfigEnvCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigEnvCommand {
    Get {
        #[arg(long)]
        env_file: Option<PathBuf>,
        key: String,
    },
    Set {
        #[arg(long)]
        env_file: Option<PathBuf>,
        key: String,
        value: String,
    },
    Unset {
        #[arg(long)]
        env_file: Option<PathBuf>,
        key: String,
    },
    List {
        #[arg(long)]
        env_file: Option<PathBuf>,
    },
}

pub fn run(command: ConfigCommand) -> anyhow::Result<()> {
    let paths = default_paths();
    match command {
        ConfigCommand::Get { config, key } => {
            let path = config.unwrap_or(paths.config_file);
            print_json(&config_get(&path, &key)?)
        }
        ConfigCommand::Set { config, key, value } => {
            let path = config.unwrap_or(paths.config_file);
            print_json(&config_set(&path, &key, &value)?)
        }
        ConfigCommand::Unset { config, key } => {
            let path = config.unwrap_or(paths.config_file);
            print_json(&config_unset(&path, &key)?)
        }
        ConfigCommand::List { config } => {
            let path = config.unwrap_or(paths.config_file);
            print_json(&config_list(&path)?)
        }
        ConfigCommand::Validate { config } => {
            let path = config.unwrap_or(paths.config_file);
            print_json(&config_validate(&path)?)
        }
        ConfigCommand::Path { config, env_file } => {
            let config = config.unwrap_or(paths.config_file);
            let env_file = env_file.unwrap_or(paths.env_file);
            print_json(&config_paths(&config, &env_file))
        }
        ConfigCommand::Env { command } => run_env(command, paths),
    }
}

fn run_env(command: ConfigEnvCommand, paths: AgentPaths) -> anyhow::Result<()> {
    match command {
        ConfigEnvCommand::Get { env_file, key } => {
            let path = env_file.unwrap_or(paths.env_file);
            print_json(&env_get(&path, &key)?)
        }
        ConfigEnvCommand::Set {
            env_file,
            key,
            value,
        } => {
            let path = env_file.unwrap_or(paths.env_file);
            print_json(&env_set(&path, &key, &value)?)
        }
        ConfigEnvCommand::Unset { env_file, key } => {
            let path = env_file.unwrap_or(paths.env_file);
            print_json(&env_unset(&path, &key)?)
        }
        ConfigEnvCommand::List { env_file } => {
            let path = env_file.unwrap_or(paths.env_file);
            print_json(&env_list(&path)?)
        }
    }
}

fn print_json(value: &impl Serialize) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
