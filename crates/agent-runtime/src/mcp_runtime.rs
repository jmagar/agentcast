use crate::{
    RuntimeCatalogSnapshot, RuntimePrompt, RuntimeResource, RuntimeResourceTemplate, RuntimeTool,
    ToolCallRequest, ToolCallResponse,
};
use agent_mcp::{McpClient, McpClientOptions, McpReadResourceResult, StdioConnection};
use agent_protocol::{McpServerConfig, McpServerId, McpToolId, McpTransportConfig, ServerStatus};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
    time::Duration,
};
use thiserror::Error;

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub struct McpRuntime {
    upstreams: BTreeMap<McpServerId, RuntimeUpstream>,
    options: RuntimeOptions,
}

#[derive(Debug)]
struct RuntimeUpstream {
    name: String,
    transport: McpTransportConfig,
    operation_timeout: Duration,
    client: Option<McpClient>,
    snapshot: RuntimeCatalogSnapshot,
    failure: Option<String>,
    consecutive_failures: AtomicU32,
    circuit_open: AtomicBool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeOptions {
    pub operation_timeout: Duration,
    pub max_catalog_items_per_kind: usize,
    pub max_response_bytes: usize,
    pub circuit_breaker_failure_threshold: u32,
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            operation_timeout: Duration::from_secs(30),
            max_catalog_items_per_kind: 256,
            max_response_bytes: 1024 * 1024,
            circuit_breaker_failure_threshold: 3,
        }
    }
}

impl McpRuntime {
    pub async fn start(configs: Vec<McpServerConfig>) -> Self {
        Self::start_with_options(configs, RuntimeOptions::default()).await
    }

    pub async fn start_with_options(
        configs: Vec<McpServerConfig>,
        options: RuntimeOptions,
    ) -> Self {
        let mut runtime = Self {
            upstreams: BTreeMap::new(),
            options,
        };
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
                    transport: config.transport,
                    operation_timeout: self.options.operation_timeout,
                    client: None,
                    snapshot: empty_snapshot(server_id, server_name, ServerStatus::Disabled),
                    failure: None,
                    consecutive_failures: AtomicU32::new(0),
                    circuit_open: AtomicBool::new(false),
                },
            );
            return;
        }

        let transport = config.transport.clone();
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
            Ok(client) => match discover_snapshot(
                &server_id,
                &server_name,
                &client,
                self.options.operation_timeout,
                self.options.max_catalog_items_per_kind,
            )
            .await
            {
                Ok(snapshot) => {
                    self.upstreams.insert(
                        server_id,
                        RuntimeUpstream {
                            name: server_name,
                            transport,
                            operation_timeout: self.options.operation_timeout,
                            client: Some(client),
                            snapshot,
                            failure: None,
                            consecutive_failures: AtomicU32::new(0),
                            circuit_open: AtomicBool::new(false),
                        },
                    );
                }
                Err(error) => {
                    self.upstreams.insert(
                        server_id.clone(),
                        RuntimeUpstream {
                            name: server_name.clone(),
                            transport,
                            operation_timeout: self.options.operation_timeout,
                            client: Some(client),
                            snapshot: empty_snapshot(
                                server_id,
                                server_name,
                                ServerStatus::Degraded,
                            ),
                            failure: Some(error.to_string()),
                            consecutive_failures: AtomicU32::new(1),
                            circuit_open: AtomicBool::new(false),
                        },
                    );
                }
            },
            Err(error) => {
                self.upstreams.insert(
                    server_id.clone(),
                    RuntimeUpstream {
                        name: server_name.clone(),
                        transport,
                        operation_timeout: self.options.operation_timeout,
                        client: None,
                        snapshot: empty_snapshot(server_id, server_name, ServerStatus::Failed),
                        failure: Some(error.to_string()),
                        consecutive_failures: AtomicU32::new(1),
                        circuit_open: AtomicBool::new(false),
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

    pub fn circuit_open(&self, server_id: &McpServerId) -> bool {
        self.upstreams
            .get(server_id)
            .is_some_and(RuntimeUpstream::circuit_open)
    }

    pub async fn reprobe(&mut self, server_id: &McpServerId) -> Result<(), RuntimeError> {
        let upstream = self
            .upstreams
            .get(server_id)
            .ok_or_else(|| RuntimeError::UnknownServer(server_id.to_string()))?;
        let config = McpServerConfig {
            id: server_id.clone(),
            name: upstream.name.clone(),
            enabled: true,
            transport: upstream.transport.clone(),
            env_keys: Vec::new(),
        };
        self.start_one(config).await;
        Ok(())
    }

    pub async fn shutdown(mut self) {
        self.upstreams.clear();
    }

    pub async fn call_tool(
        &self,
        request: ToolCallRequest,
    ) -> Result<ToolCallResponse, RuntimeError> {
        let upstream = self
            .upstreams
            .get(&request.server_id)
            .ok_or_else(|| RuntimeError::UnknownServer(request.server_id.to_string()))?;
        upstream.ensure_circuit_closed()?;
        let client = upstream
            .client
            .as_ref()
            .ok_or_else(|| RuntimeError::ServerUnavailable(upstream.name.clone()))?;
        let result = tokio::time::timeout(
            upstream.operation_timeout,
            call_tool_with_client(upstream, client, request),
        )
        .await
        .map_err(|_| RuntimeError::OperationTimedOut(upstream.name.clone()))
        .and_then(|result| result.map_err(|error| RuntimeError::Mcp(error.to_string())));
        let output = result.and_then(|result| {
            let output = serde_json::to_value(result).unwrap_or(Value::Null);
            ensure_response_size(&output, self.options.max_response_bytes).map(|()| output)
        });
        let output =
            upstream.record_result(output, self.options.circuit_breaker_failure_threshold)?;
        Ok(ToolCallResponse { output })
    }

    pub async fn call_tool_with_bearer(
        &self,
        mut request: ToolCallRequest,
        bearer_token: impl Into<String>,
    ) -> Result<ToolCallResponse, RuntimeError> {
        request.auth = Some(crate::RuntimeRequestAuth::bearer(bearer_token));
        self.call_tool(request).await
    }

    pub async fn read_resource_with_bearer(
        &self,
        server_id: &McpServerId,
        uri: &str,
        bearer_token: impl Into<String>,
    ) -> Result<McpReadResourceResult, RuntimeError> {
        let upstream = self
            .upstreams
            .get(server_id)
            .ok_or_else(|| RuntimeError::UnknownServer(server_id.to_string()))?;
        upstream.ensure_circuit_closed()?;
        let client = upstream
            .client
            .as_ref()
            .ok_or_else(|| RuntimeError::ServerUnavailable(upstream.name.clone()))?;
        let auth = crate::RuntimeRequestAuth::bearer(bearer_token);
        let result = tokio::time::timeout(
            upstream.operation_timeout,
            read_resource_with_client(upstream, client, uri, Some(&auth)),
        )
        .await
        .map_err(|_| RuntimeError::OperationTimedOut(upstream.name.clone()))
        .and_then(|result| result.map_err(|error| RuntimeError::Mcp(error.to_string())));
        let result =
            upstream.record_result(result, self.options.circuit_breaker_failure_threshold)?;
        ensure_response_size(
            &serde_json::to_value(&result).unwrap_or(Value::Null),
            self.options.max_response_bytes,
        )?;
        Ok(result)
    }

    pub async fn get_prompt_with_bearer(
        &self,
        server_id: &McpServerId,
        name: &str,
        arguments: Option<serde_json::Map<String, Value>>,
        bearer_token: impl Into<String>,
    ) -> Result<Value, RuntimeError> {
        let upstream = self
            .upstreams
            .get(server_id)
            .ok_or_else(|| RuntimeError::UnknownServer(server_id.to_string()))?;
        upstream.ensure_circuit_closed()?;
        let client = upstream
            .client
            .as_ref()
            .ok_or_else(|| RuntimeError::ServerUnavailable(upstream.name.clone()))?;
        let auth = crate::RuntimeRequestAuth::bearer(bearer_token);
        let result = tokio::time::timeout(
            upstream.operation_timeout,
            get_prompt_with_client(upstream, client, name, arguments, Some(&auth)),
        )
        .await
        .map_err(|_| RuntimeError::OperationTimedOut(upstream.name.clone()))
        .and_then(|result| result.map_err(|error| RuntimeError::Mcp(error.to_string())));
        let result =
            upstream.record_result(result, self.options.circuit_breaker_failure_threshold)?;
        ensure_response_size(&result, self.options.max_response_bytes)?;
        Ok(result)
    }

    pub fn would_use_ephemeral_http_auth(
        &self,
        server_id: &McpServerId,
        auth: Option<&crate::RuntimeRequestAuth>,
    ) -> bool {
        self.upstreams
            .get(server_id)
            .is_some_and(|upstream| ephemeral_http_auth(&upstream.transport, auth).is_some())
    }
}

async fn call_tool_with_client(
    upstream: &RuntimeUpstream,
    client: &McpClient,
    request: ToolCallRequest,
) -> Result<agent_mcp::McpToolResult, agent_mcp::McpError> {
    if let Some((url, bearer_token)) =
        ephemeral_http_auth(&upstream.transport, request.auth.as_ref())
    {
        let client = McpClient::connect_streamable_http(
            url,
            Some(bearer_token),
            McpClientOptions::default(),
        )
        .await?;
        return client
            .call_tool(request.tool_id.as_str(), request.arguments)
            .await;
    }

    client
        .call_tool(request.tool_id.as_str(), request.arguments)
        .await
}

async fn read_resource_with_client(
    upstream: &RuntimeUpstream,
    client: &McpClient,
    uri: &str,
    auth: Option<&crate::RuntimeRequestAuth>,
) -> Result<McpReadResourceResult, agent_mcp::McpError> {
    if let Some((url, bearer_token)) = ephemeral_http_auth(&upstream.transport, auth) {
        let client = McpClient::connect_streamable_http(
            url,
            Some(bearer_token),
            McpClientOptions::default(),
        )
        .await?;
        return client.read_resource(uri).await;
    }

    client.read_resource(uri).await
}

async fn get_prompt_with_client(
    upstream: &RuntimeUpstream,
    client: &McpClient,
    name: &str,
    arguments: Option<serde_json::Map<String, Value>>,
    auth: Option<&crate::RuntimeRequestAuth>,
) -> Result<Value, agent_mcp::McpError> {
    if let Some((url, bearer_token)) = ephemeral_http_auth(&upstream.transport, auth) {
        let client = McpClient::connect_streamable_http(
            url,
            Some(bearer_token),
            McpClientOptions::default(),
        )
        .await?;
        return client.get_prompt(name, arguments).await;
    }

    client.get_prompt(name, arguments).await
}

fn ephemeral_http_auth(
    transport: &McpTransportConfig,
    auth: Option<&crate::RuntimeRequestAuth>,
) -> Option<(String, String)> {
    match (transport, auth) {
        (McpTransportConfig::StreamableHttp { url, .. }, Some(auth)) => {
            Some((url.clone(), auth.bearer_token.clone()))
        }
        _ => None,
    }
}

impl McpRuntime {
    pub async fn read_resource(
        &self,
        server_id: &McpServerId,
        uri: &str,
    ) -> Result<McpReadResourceResult, RuntimeError> {
        let upstream = self
            .upstreams
            .get(server_id)
            .ok_or_else(|| RuntimeError::UnknownServer(server_id.to_string()))?;
        upstream.ensure_circuit_closed()?;
        let client = upstream
            .client
            .as_ref()
            .ok_or_else(|| RuntimeError::ServerUnavailable(upstream.name.clone()))?;
        let result = tokio::time::timeout(
            upstream.operation_timeout,
            read_resource_with_client(upstream, client, uri, None),
        )
        .await
        .map_err(|_| RuntimeError::OperationTimedOut(upstream.name.clone()))
        .and_then(|result| result.map_err(|error| RuntimeError::Mcp(error.to_string())));
        let result =
            upstream.record_result(result, self.options.circuit_breaker_failure_threshold)?;
        ensure_response_size(
            &serde_json::to_value(&result).unwrap_or(Value::Null),
            self.options.max_response_bytes,
        )?;
        Ok(result)
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
        upstream.ensure_circuit_closed()?;
        let client = upstream
            .client
            .as_ref()
            .ok_or_else(|| RuntimeError::ServerUnavailable(upstream.name.clone()))?;
        let result = tokio::time::timeout(
            upstream.operation_timeout,
            get_prompt_with_client(upstream, client, name, arguments, None),
        )
        .await
        .map_err(|_| RuntimeError::OperationTimedOut(upstream.name.clone()))
        .and_then(|result| result.map_err(|error| RuntimeError::Mcp(error.to_string())));
        let result =
            upstream.record_result(result, self.options.circuit_breaker_failure_threshold)?;
        ensure_response_size(&result, self.options.max_response_bytes)?;
        Ok(result)
    }
}

async fn discover_snapshot(
    server_id: &McpServerId,
    server_name: &str,
    client: &McpClient,
    operation_timeout: Duration,
    max_items_per_kind: usize,
) -> Result<RuntimeCatalogSnapshot, RuntimeError> {
    let mut tools = tokio::time::timeout(operation_timeout, client.list_tools())
        .await
        .map_err(|_| RuntimeError::OperationTimedOut(server_name.to_string()))?
        .map_err(|error| RuntimeError::Mcp(error.to_string()))?
        .into_iter()
        .map(|tool| RuntimeTool {
            id: McpToolId::new(tool.name.clone()),
            name: tool.name,
            title: tool.title,
            description: tool.description,
            input_schema: tool.input_schema,
            output_schema: tool.output_schema,
            annotations: tool.annotations,
        })
        .collect();
    cap_vec(&mut tools, max_items_per_kind);
    let mut resources = tokio::time::timeout(operation_timeout, client.list_resources())
        .await
        .map_err(|_| RuntimeError::OperationTimedOut(server_name.to_string()))?
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
    cap_vec(&mut resources, max_items_per_kind);
    let mut resource_templates =
        tokio::time::timeout(operation_timeout, client.list_resource_templates())
            .await
            .map_err(|_| RuntimeError::OperationTimedOut(server_name.to_string()))?
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
    cap_vec(&mut resource_templates, max_items_per_kind);
    let mut prompts = tokio::time::timeout(operation_timeout, client.list_prompts())
        .await
        .map_err(|_| RuntimeError::OperationTimedOut(server_name.to_string()))?
        .map_err(|error| RuntimeError::Mcp(error.to_string()))?
        .into_iter()
        .map(|prompt| RuntimePrompt {
            name: prompt.name,
            title: prompt.title,
            description: prompt.description,
            arguments: prompt.arguments,
        })
        .collect();
    cap_vec(&mut prompts, max_items_per_kind);

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

impl RuntimeUpstream {
    fn circuit_open(&self) -> bool {
        self.circuit_open.load(Ordering::Relaxed)
    }

    fn ensure_circuit_closed(&self) -> Result<(), RuntimeError> {
        if self.circuit_open() {
            Err(RuntimeError::CircuitOpen(self.name.clone()))
        } else {
            Ok(())
        }
    }

    fn record_result<T>(
        &self,
        result: Result<T, RuntimeError>,
        failure_threshold: u32,
    ) -> Result<T, RuntimeError> {
        match result {
            Ok(value) => {
                self.consecutive_failures.store(0, Ordering::Relaxed);
                self.circuit_open.store(false, Ordering::Relaxed);
                Ok(value)
            }
            Err(error) => {
                let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
                if failure_threshold > 0 && failures >= failure_threshold {
                    self.circuit_open.store(true, Ordering::Relaxed);
                }
                Err(error)
            }
        }
    }
}

fn ensure_response_size(value: &Value, max_response_bytes: usize) -> Result<(), RuntimeError> {
    if max_response_bytes == 0 {
        return Ok(());
    }
    let size = serde_json::to_vec(value)
        .map(|bytes| bytes.len())
        .unwrap_or(0);
    if size > max_response_bytes {
        Err(RuntimeError::ResponseTooLarge {
            size,
            limit: max_response_bytes,
        })
    } else {
        Ok(())
    }
}

fn cap_vec<T>(items: &mut Vec<T>, max_items: usize) {
    if max_items > 0 && items.len() > max_items {
        items.truncate(max_items);
    }
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
    #[error("MCP server `{0}` operation timed out")]
    OperationTimedOut(String),
    #[error("MCP server `{0}` circuit is open")]
    CircuitOpen(String),
    #[error("MCP response is too large: {size} bytes exceeds {limit} bytes")]
    ResponseTooLarge { size: usize, limit: usize },
    #[error("MCP runtime error: {0}")]
    Mcp(String),
}
