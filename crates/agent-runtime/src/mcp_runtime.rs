use crate::{
    RuntimeCatalogSnapshot, RuntimePrompt, RuntimeResource, RuntimeResourceTemplate, RuntimeTool,
    ToolCallRequest, ToolCallResponse,
};
use agent_mcp::{McpClient, McpClientOptions, McpReadResourceResult, StdioConnection};
use agent_protocol::{McpServerConfig, McpServerId, McpToolId, McpTransportConfig, ServerStatus};
use futures::StreamExt;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    sync::Arc,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
    time::{Duration, Instant},
};
use thiserror::Error;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[cfg(test)]
mod tests;

const MAX_CONCURRENT_STARTUPS: usize = 8;
const PROTECTED_HTTP_CLIENT_TTL: Duration = Duration::from_secs(5 * 60);

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
    lifecycle_token: CancellationToken,
    protected_http_clients: Mutex<BTreeMap<String, ProtectedHttpClient>>,
    snapshot: RuntimeCatalogSnapshot,
    failure: Option<String>,
    consecutive_failures: AtomicU32,
    circuit_open: AtomicBool,
}

struct RuntimeUpstreamInit {
    name: String,
    transport: McpTransportConfig,
    operation_timeout: Duration,
    client: Option<McpClient>,
    lifecycle_token: CancellationToken,
    snapshot: RuntimeCatalogSnapshot,
    failure: Option<String>,
    consecutive_failures: u32,
}

#[derive(Debug)]
struct ProtectedHttpClient {
    client: Arc<McpClient>,
    lifecycle_token: CancellationToken,
    created_at: Instant,
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
        let runtime_options = runtime.options;
        let mut startups = futures::stream::iter(configs.into_iter().map(|config| async move {
            let server_id = config.id.clone();
            (server_id, start_upstream(config, runtime_options).await)
        }))
        .buffer_unordered(MAX_CONCURRENT_STARTUPS);

        while let Some((server_id, upstream)) = futures::StreamExt::next(&mut startups).await {
            runtime.upstreams.insert(server_id, upstream);
        }
        runtime
    }

    pub async fn start_one(&mut self, config: McpServerConfig) {
        let server_id = config.id.clone();
        let upstream = start_upstream(config, self.options).await;
        self.upstreams.insert(server_id, upstream);
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
        for upstream in self.upstreams.values() {
            upstream.cancel_lifecycle().await;
        }
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

async fn start_upstream(config: McpServerConfig, options: RuntimeOptions) -> RuntimeUpstream {
    let server_id = config.id.clone();
    let server_name = config.name.clone();
    if !config.enabled {
        return RuntimeUpstream::new(
            server_name.clone(),
            config.transport,
            options.operation_timeout,
            None,
            empty_snapshot(server_id, server_name, ServerStatus::Disabled),
            None,
            0,
        );
    }

    let transport = config.transport.clone();
    let lifecycle_token = CancellationToken::new();
    let client = connect_client(config.transport, lifecycle_token.clone()).await;

    match client {
        Ok(client) => {
            let (snapshot, failure) = discover_snapshot(
                &server_id,
                &server_name,
                &client,
                options.operation_timeout,
                options.max_catalog_items_per_kind,
            )
            .await;
            let failures = u32::from(failure.is_some());
            RuntimeUpstream::new_with_token(RuntimeUpstreamInit {
                name: server_name,
                transport,
                operation_timeout: options.operation_timeout,
                client: Some(client),
                lifecycle_token,
                snapshot,
                failure,
                consecutive_failures: failures,
            })
        }
        Err(error) => RuntimeUpstream::new_with_token(RuntimeUpstreamInit {
            name: server_name.clone(),
            transport,
            operation_timeout: options.operation_timeout,
            client: None,
            lifecycle_token,
            snapshot: empty_snapshot(server_id, server_name, ServerStatus::Failed),
            failure: Some(error.to_string()),
            consecutive_failures: 1,
        }),
    }
}

async fn connect_client(
    transport: McpTransportConfig,
    lifecycle_token: CancellationToken,
) -> agent_mcp::McpResult<McpClient> {
    let options = McpClientOptions {
        cancellation_token: lifecycle_token,
    };
    match transport {
        McpTransportConfig::Stdio { command, args, env } => {
            let connection = StdioConnection::new(command, args).with_env(env);
            McpClient::connect_stdio(connection, options).await
        }
        McpTransportConfig::StreamableHttp {
            url,
            bearer_token_env,
        } => {
            let bearer_token = bearer_token_env.and_then(|key| std::env::var(key).ok());
            McpClient::connect_streamable_http(url, bearer_token, options).await
        }
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
        let cache_key = protected_http_cache_key(&url, &bearer_token);
        let mut clients = upstream.protected_http_clients.lock().await;
        let client = protected_http_client(&mut clients, cache_key, url, bearer_token).await?;
        drop(clients);
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
        let cache_key = protected_http_cache_key(&url, &bearer_token);
        let mut clients = upstream.protected_http_clients.lock().await;
        let client = protected_http_client(&mut clients, cache_key, url, bearer_token).await?;
        drop(clients);
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
        let cache_key = protected_http_cache_key(&url, &bearer_token);
        let mut clients = upstream.protected_http_clients.lock().await;
        let client = protected_http_client(&mut clients, cache_key, url, bearer_token).await?;
        drop(clients);
        return client.get_prompt(name, arguments).await;
    }

    client.get_prompt(name, arguments).await
}

async fn protected_http_client(
    clients: &mut BTreeMap<String, ProtectedHttpClient>,
    cache_key: String,
    url: String,
    bearer_token: String,
) -> Result<Arc<McpClient>, agent_mcp::McpError> {
    let expired = clients
        .get(&cache_key)
        .is_some_and(|client| client.created_at.elapsed() >= PROTECTED_HTTP_CLIENT_TTL);
    if expired && let Some(client) = clients.remove(&cache_key) {
        client.lifecycle_token.cancel();
    }

    if !clients.contains_key(&cache_key) {
        let lifecycle_token = CancellationToken::new();
        let client = McpClient::connect_streamable_http(
            url,
            Some(bearer_token),
            McpClientOptions {
                cancellation_token: lifecycle_token.clone(),
            },
        )
        .await?;
        clients.insert(
            cache_key.clone(),
            ProtectedHttpClient {
                client: Arc::new(client),
                lifecycle_token,
                created_at: Instant::now(),
            },
        );
    }

    Ok(clients
        .get(&cache_key)
        .expect("protected HTTP client inserted")
        .client
        .clone())
}

fn protected_http_cache_key(url: &str, bearer_token: &str) -> String {
    format!("{url}\0{bearer_token}")
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
) -> (RuntimeCatalogSnapshot, Option<String>) {
    let (tools, resources, resource_templates, prompts) = tokio::join!(
        discover_tools(server_name, client, operation_timeout, max_items_per_kind),
        discover_resources(server_name, client, operation_timeout, max_items_per_kind),
        discover_resource_templates(server_name, client, operation_timeout, max_items_per_kind),
        discover_prompts(server_name, client, operation_timeout, max_items_per_kind),
    );

    let mut failures = Vec::new();
    let tools = collect_catalog_result("tools", tools, &mut failures);
    let resources = collect_catalog_result("resources", resources, &mut failures);
    let resource_templates =
        collect_catalog_result("resource_templates", resource_templates, &mut failures);
    let prompts = collect_catalog_result("prompts", prompts, &mut failures);
    let status = if failures.is_empty() {
        ServerStatus::Healthy
    } else {
        ServerStatus::Degraded
    };

    (
        RuntimeCatalogSnapshot {
            server_id: server_id.clone(),
            server_name: server_name.to_string(),
            status,
            tools,
            resources,
            resource_templates,
            prompts,
        },
        (!failures.is_empty()).then(|| failures.join("; ")),
    )
}

async fn discover_tools(
    server_name: &str,
    client: &McpClient,
    operation_timeout: Duration,
    max_items_per_kind: usize,
) -> Result<Vec<RuntimeTool>, RuntimeError> {
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
    Ok(tools)
}

async fn discover_resources(
    server_name: &str,
    client: &McpClient,
    operation_timeout: Duration,
    max_items_per_kind: usize,
) -> Result<Vec<RuntimeResource>, RuntimeError> {
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
    Ok(resources)
}

async fn discover_resource_templates(
    server_name: &str,
    client: &McpClient,
    operation_timeout: Duration,
    max_items_per_kind: usize,
) -> Result<Vec<RuntimeResourceTemplate>, RuntimeError> {
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
    Ok(resource_templates)
}

async fn discover_prompts(
    server_name: &str,
    client: &McpClient,
    operation_timeout: Duration,
    max_items_per_kind: usize,
) -> Result<Vec<RuntimePrompt>, RuntimeError> {
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
    Ok(prompts)
}

fn collect_catalog_result<T>(
    kind: &str,
    result: Result<Vec<T>, RuntimeError>,
    failures: &mut Vec<String>,
) -> Vec<T> {
    match result {
        Ok(items) => items,
        Err(error) => {
            failures.push(format!("{kind}: {error}"));
            Vec::new()
        }
    }
}

impl RuntimeUpstream {
    fn new(
        name: String,
        transport: McpTransportConfig,
        operation_timeout: Duration,
        client: Option<McpClient>,
        snapshot: RuntimeCatalogSnapshot,
        failure: Option<String>,
        consecutive_failures: u32,
    ) -> Self {
        Self::new_with_token(RuntimeUpstreamInit {
            name,
            transport,
            operation_timeout,
            client,
            lifecycle_token: CancellationToken::new(),
            snapshot,
            failure,
            consecutive_failures,
        })
    }

    fn new_with_token(init: RuntimeUpstreamInit) -> Self {
        Self {
            name: init.name,
            transport: init.transport,
            operation_timeout: init.operation_timeout,
            client: init.client,
            lifecycle_token: init.lifecycle_token,
            protected_http_clients: Mutex::new(BTreeMap::new()),
            snapshot: init.snapshot,
            failure: init.failure,
            consecutive_failures: AtomicU32::new(init.consecutive_failures),
            circuit_open: AtomicBool::new(false),
        }
    }

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

    async fn cancel_lifecycle(&self) {
        self.lifecycle_token.cancel();
        for client in self.protected_http_clients.lock().await.values() {
            client.lifecycle_token.cancel();
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
