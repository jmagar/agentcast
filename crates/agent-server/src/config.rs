use agent_config::{discover_known_mcp_configs, parse_mcp_json};
use agent_protocol::McpServerConfig;
use std::path::PathBuf;

pub(crate) fn load_mcp_configs_from(
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

#[cfg(test)]
mod tests;
