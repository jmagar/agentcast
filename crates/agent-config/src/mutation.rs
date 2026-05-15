use crate::{AgentConfig, ConfigResult, McpUpstreamConfig};

#[cfg(test)]
mod tests;

pub fn add_or_replace_upstream(
    config: &mut AgentConfig,
    upstream: McpUpstreamConfig,
) -> ConfigResult<Option<McpUpstreamConfig>> {
    upstream.validate()?;
    Ok(config.mcp.upstreams.insert(upstream.id.clone(), upstream))
}

pub fn remove_upstream(config: &mut AgentConfig, id: &str) -> Option<McpUpstreamConfig> {
    config.mcp.upstreams.remove(id)
}
