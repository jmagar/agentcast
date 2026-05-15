use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{ConfigError, ConfigResult};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(default)]
    pub upstreams: BTreeMap<String, McpUpstreamConfig>,
}

impl McpConfig {
    pub fn validate(&mut self) -> ConfigResult<()> {
        for (id, upstream) in &mut self.upstreams {
            validate_upstream_id(id)?;
            upstream.id = id.clone();
            upstream.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpUpstreamConfig {
    #[serde(skip)]
    pub id: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(flatten)]
    pub transport: McpTransport,
}

impl McpUpstreamConfig {
    pub fn new(id: impl Into<String>, transport: McpTransport) -> ConfigResult<Self> {
        let id = id.into();
        validate_upstream_id(&id)?;
        let upstream = Self {
            id,
            enabled: true,
            transport,
        };
        upstream.validate()?;
        Ok(upstream)
    }

    pub fn validate(&self) -> ConfigResult<()> {
        match &self.transport {
            McpTransport::Stdio(config) => config.validate(),
            McpTransport::StreamableHttp(config) => config.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub enum McpTransport {
    Stdio(StdioUpstreamConfig),
    StreamableHttp(StreamableHttpUpstreamConfig),
}

impl Default for McpTransport {
    fn default() -> Self {
        Self::Stdio(StdioUpstreamConfig::default())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StdioUpstreamConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub cwd: Option<PathBuf>,
    #[serde(default)]
    pub env: BTreeMap<String, EnvBinding>,
}

impl StdioUpstreamConfig {
    fn validate(&self) -> ConfigResult<()> {
        if self.command.trim().is_empty() {
            return Err(ConfigError::InvalidConfig(
                "stdio upstream command cannot be blank".into(),
            ));
        }
        for key in self.env.keys() {
            validate_env_key(key)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamableHttpUpstreamConfig {
    pub url: String,
    #[serde(default)]
    pub bearer_token_env: Option<String>,
}

impl StreamableHttpUpstreamConfig {
    fn validate(&self) -> ConfigResult<()> {
        let url = url::Url::parse(&self.url)
            .map_err(|error| ConfigError::InvalidConfig(format!("invalid MCP URL: {error}")))?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err(ConfigError::InvalidConfig(
                "streamable HTTP upstream URL must use http or https".into(),
            ));
        }
        if let Some(key) = &self.bearer_token_env {
            validate_env_key(key)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum EnvBinding {
    Config { value: String },
    Env { key: String },
}

fn default_enabled() -> bool {
    true
}

fn validate_upstream_id(id: &str) -> ConfigResult<()> {
    let valid = !id.trim().is_empty()
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'));
    if valid {
        Ok(())
    } else {
        Err(ConfigError::InvalidConfig(format!(
            "upstream id `{id}` must be non-empty and contain only ASCII alphanumerics, dash, underscore, or dot"
        )))
    }
}

fn validate_env_key(key: &str) -> ConfigResult<()> {
    let valid = !key.is_empty()
        && key.starts_with(|ch: char| ch.is_ascii_uppercase())
        && key
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_');
    if valid {
        Ok(())
    } else {
        Err(ConfigError::InvalidConfig(format!(
            "environment key `{key}` must match ^[A-Z][A-Z0-9_]*$"
        )))
    }
}
