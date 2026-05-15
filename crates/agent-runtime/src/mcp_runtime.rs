use crate::{
    RuntimeCatalogSnapshot, RuntimePrompt, RuntimeResource, RuntimeResourceTemplate, RuntimeTool,
    ToolCallRequest, ToolCallResponse,
};
use agent_mcp::{McpClient, McpClientOptions, McpReadResourceResult, StdioConnection};
use agent_protocol::{McpServerConfig, McpServerId, McpToolId, McpTransportConfig, ServerStatus};
use serde_json::Value;
use std::collections::BTreeMap;
use thiserror::Error;

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub struct McpRuntime {
    upstreams: BTreeMap<McpServerId, RuntimeUpstream>,
}

#[derive(Debug)]
struct RuntimeUpstream {
    name: String,
    client: Option<McpClient>,
    snapshot: RuntimeCatalogSnapshot,
    failure: Option<String>,
}

impl McpRuntime {
    pub async fn start(configs: Vec<McpServerConfig>) -> Self {
        let mut runtime = Self::default();
        for config in configs {
            runtime.start_one(config).await;
        }
        runtime
    }

    pub async fn start_one(&mut self, config: McpServerConfig) {
        let server_id = config.id.clone();
        let server_name = config.name.clone();
        if !config.enabled {
            self.upstreams.insert(
                server_id.clone(),
                RuntimeUpstream {
                    name: server_name.clone(),
                    client: None,
                    snapshot: empty_snapshot(server_id, server_name, ServerStatus::Disabled),
                    failure: None,
                },
            );
            return;
        }

        let client = match config.transport {
            McpTransportConfig::Stdio { command, args, env } => {
                let connection = StdioConnection::new(command, args).with_env(env);
                McpClient::connect_stdio(connection, McpClientOptions::default()).await
            }
            McpTransportConfig::StreamableHttp {
                url,
                bearer_token_env,
            } => {
                let bearer_token = bearer_token_env.and_then(|key| std::env::var(key).ok());
                McpClient::connect_streamable_http(url, bearer_token, McpClientOptions::default())
                    .await
            }
        };

        match client {
            Ok(client) => match discover_snapshot(&server_id, &server_name, &client).await {
                Ok(snapshot) => {
                    self.upstreams.insert(
                        server_id,
                        RuntimeUpstream {
                            name: server_name,
                            client: Some(client),
                            snapshot,
                            failure: None,
                        },
                    );
                }
                Err(error) => {
                    self.upstreams.insert(
                        server_id.clone(),
                        RuntimeUpstream {
                            name: server_name.clone(),
                            client: Some(client),
                            snapshot: empty_snapshot(
                                server_id,
                                server_name,
                                ServerStatus::Degraded,
                            ),
                            failure: Some(error.to_string()),
                        },
                    );
                }
            },
            Err(error) => {
                self.upstreams.insert(
                    server_id.clone(),
                    RuntimeUpstream {
                        name: server_name.clone(),
                        client: None,
                        snapshot: empty_snapshot(server_id, server_name, ServerStatus::Failed),
                        failure: Some(error.to_string()),
                    },
                );
            }
        }
    }

    pub fn snapshots(&self) -> Vec<RuntimeCatalogSnapshot> {
        self.upstreams
            .values()
            .map(|upstream| upstream.snapshot.clone())
            .collect()
    }

    pub fn failure(&self, server_id: &McpServerId) -> Option<&str> {
        self.upstreams
            .get(server_id)
            .and_then(|upstream| upstream.failure.as_deref())
    }

    pub async fn call_tool(
        &self,
        request: ToolCallRequest,
    ) -> Result<ToolCallResponse, RuntimeError> {
        let upstream = self
            .upstreams
            .get(&request.server_id)
            .ok_or_else(|| RuntimeError::UnknownServer(request.server_id.to_string()))?;
        let client = upstream
            .client
            .as_ref()
            .ok_or_else(|| RuntimeError::ServerUnavailable(upstream.name.clone()))?;
        let result = client
            .call_tool(request.tool_id.as_str(), request.arguments)
            .await
            .map_err(|error| RuntimeError::Mcp(error.to_string()))?;
        Ok(ToolCallResponse {
            output: serde_json::to_value(result).unwrap_or(Value::Null),
        })
    }

    pub async fn read_resource(
        &self,
        server_id: &McpServerId,
        uri: &str,
    ) -> Result<McpReadResourceResult, RuntimeError> {
        let upstream = self
            .upstreams
            .get(server_id)
            .ok_or_else(|| RuntimeError::UnknownServer(server_id.to_string()))?;
        let client = upstream
            .client
            .as_ref()
            .ok_or_else(|| RuntimeError::ServerUnavailable(upstream.name.clone()))?;
        client
            .read_resource(uri)
            .await
            .map_err(|error| RuntimeError::Mcp(error.to_string()))
    }

    pub async fn get_prompt(
        &self,
        server_id: &McpServerId,
        name: &str,
        arguments: Option<serde_json::Map<String, Value>>,
    ) -> Result<Value, RuntimeError> {
        let upstream = self
            .upstreams
            .get(server_id)
            .ok_or_else(|| RuntimeError::UnknownServer(server_id.to_string()))?;
        let client = upstream
            .client
            .as_ref()
            .ok_or_else(|| RuntimeError::ServerUnavailable(upstream.name.clone()))?;
        client
            .get_prompt(name, arguments)
            .await
            .map_err(|error| RuntimeError::Mcp(error.to_string()))
    }
}

async fn discover_snapshot(
    server_id: &McpServerId,
    server_name: &str,
    client: &McpClient,
) -> Result<RuntimeCatalogSnapshot, RuntimeError> {
    let tools = client
        .list_tools()
        .await
        .map_err(|error| RuntimeError::Mcp(error.to_string()))?
        .into_iter()
        .map(|tool| RuntimeTool {
            id: McpToolId::new(tool.name.clone()),
            name: tool.name,
            title: tool.title,
            description: tool.description,
            input_schema: tool.input_schema,
        })
        .collect();
    let resources = client
        .list_resources()
        .await
        .map_err(|error| RuntimeError::Mcp(error.to_string()))?
        .into_iter()
        .map(|resource| RuntimeResource {
            uri: resource.uri,
            name: resource.name,
            title: resource.title,
            description: resource.description,
            mime_type: resource.mime_type,
            size: resource.size,
        })
        .collect();
    let resource_templates = client
        .list_resource_templates()
        .await
        .map_err(|error| RuntimeError::Mcp(error.to_string()))?
        .into_iter()
        .map(|template| RuntimeResourceTemplate {
            uri_template: template.uri_template,
            name: template.name,
            title: template.title,
            description: template.description,
            mime_type: template.mime_type,
        })
        .collect();
    let prompts = client
        .list_prompts()
        .await
        .map_err(|error| RuntimeError::Mcp(error.to_string()))?
        .into_iter()
        .map(|prompt| RuntimePrompt {
            name: prompt.name,
            title: prompt.title,
            description: prompt.description,
            arguments: prompt.arguments,
        })
        .collect();

    Ok(RuntimeCatalogSnapshot {
        server_id: server_id.clone(),
        server_name: server_name.to_string(),
        status: ServerStatus::Healthy,
        tools,
        resources,
        resource_templates,
        prompts,
    })
}

fn empty_snapshot(
    server_id: McpServerId,
    server_name: String,
    status: ServerStatus,
) -> RuntimeCatalogSnapshot {
    RuntimeCatalogSnapshot {
        server_id,
        server_name,
        status,
        tools: Vec::new(),
        resources: Vec::new(),
        resource_templates: Vec::new(),
        prompts: Vec::new(),
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RuntimeError {
    #[error("unknown MCP server `{0}`")]
    UnknownServer(String),
    #[error("MCP server `{0}` is unavailable")]
    ServerUnavailable(String),
    #[error("MCP runtime error: {0}")]
    Mcp(String),
}
