use agent_auth::{
    AuthDecision, BearerTokenVerifier, FixtureBearerTokenVerifier, ProtectedResourceMetadata,
};
use agent_gateway::{ProtectedRouteIndex, ProtectedRouteTarget};
use agent_protocol::{McpServerId, McpToolId};
use agent_runtime::{
    McpRuntime, RuntimeCatalogSnapshot, RuntimeError, RuntimeRequestAuth, ToolCallRequest,
};
use serde_json::{Map, Value, json};
use std::sync::Arc;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct ProtectedMcpRouteApi {
    routes: ProtectedRouteIndex,
    verifier: Arc<dyn BearerTokenVerifier>,
}

impl ProtectedMcpRouteApi {
    pub fn new(routes: ProtectedRouteIndex) -> Self {
        Self::new_with_verifier(routes, Arc::new(FixtureBearerTokenVerifier))
    }

    pub fn new_with_verifier(
        routes: ProtectedRouteIndex,
        verifier: Arc<dyn BearerTokenVerifier>,
    ) -> Self {
        Self { routes, verifier }
    }

    pub fn handle(&self, request: ProtectedMcpRequest) -> ProtectedMcpResponse {
        if let Some(route) = self.routes.resolve_metadata(&request.host, &request.path) {
            return ProtectedMcpResponse::Metadata {
                status: ResponseStatus::Ok,
                metadata: route.protected_resource_metadata(),
            };
        }

        let Some(route) = self.routes.resolve(&request.host, &request.path) else {
            return ProtectedMcpResponse::NotFound {
                status: ResponseStatus::NotFound,
            };
        };

        let claims = request
            .authorization
            .as_deref()
            .and_then(|header| self.verifier.verify(header).ok());

        match route.authorize(claims.as_ref(), &request.public_origin) {
            AuthDecision::Authorized(subject) => ProtectedMcpResponse::DispatchAllowed {
                status: ResponseStatus::Accepted,
                subject: subject.subject,
                target: route.target.clone(),
            },
            AuthDecision::Unauthorized(challenge) => ProtectedMcpResponse::Challenge {
                status: ResponseStatus::Unauthorized,
                www_authenticate: challenge.www_authenticate(),
            },
            AuthDecision::Forbidden(challenge) => ProtectedMcpResponse::Challenge {
                status: ResponseStatus::Forbidden,
                www_authenticate: challenge.www_authenticate(),
            },
        }
    }

    pub async fn handle_json_rpc(
        &self,
        runtime: &McpRuntime,
        request: ProtectedMcpJsonRpcRequest,
    ) -> ProtectedMcpJsonRpcResponse {
        let authorization = self.handle(ProtectedMcpRequest {
            host: request.host,
            path: request.path,
            public_origin: request.public_origin,
            authorization: request.authorization,
        });

        let ProtectedMcpResponse::DispatchAllowed { target, .. } = authorization else {
            return ProtectedMcpJsonRpcResponse::Rejected(authorization);
        };

        let ProtectedRouteTarget::UpstreamMcp { server_id } = target;
        ProtectedMcpJsonRpcResponse::JsonRpc(
            dispatch_json_rpc(runtime, &server_id, request.body, None).await,
        )
    }

    pub async fn handle_json_rpc_with_upstream_credential(
        &self,
        runtime: &McpRuntime,
        request: ProtectedMcpJsonRpcRequest,
        upstream_access_token: Option<String>,
    ) -> ProtectedMcpJsonRpcResponse {
        let authorization = self.handle(ProtectedMcpRequest {
            host: request.host,
            path: request.path,
            public_origin: request.public_origin,
            authorization: request.authorization,
        });

        let ProtectedMcpResponse::DispatchAllowed { target, .. } = authorization else {
            return ProtectedMcpJsonRpcResponse::Rejected(authorization);
        };

        let ProtectedRouteTarget::UpstreamMcp { server_id } = target;
        let auth = upstream_access_token.map(RuntimeRequestAuth::bearer);
        ProtectedMcpJsonRpcResponse::JsonRpc(
            dispatch_json_rpc(runtime, &server_id, request.body, auth.as_ref()).await,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedMcpRequest {
    pub host: String,
    pub path: String,
    pub public_origin: String,
    pub authorization: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProtectedMcpJsonRpcRequest {
    pub host: String,
    pub path: String,
    pub public_origin: String,
    pub authorization: Option<String>,
    pub body: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProtectedMcpJsonRpcResponse {
    JsonRpc(Value),
    Rejected(ProtectedMcpResponse),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProtectedMcpResponse {
    Metadata {
        status: ResponseStatus,
        metadata: ProtectedResourceMetadata,
    },
    DispatchAllowed {
        status: ResponseStatus,
        subject: String,
        target: ProtectedRouteTarget,
    },
    Challenge {
        status: ResponseStatus,
        www_authenticate: String,
    },
    NotFound {
        status: ResponseStatus,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseStatus {
    Ok,
    Accepted,
    Unauthorized,
    Forbidden,
    NotFound,
}

async fn dispatch_json_rpc(
    runtime: &McpRuntime,
    server_id: &McpServerId,
    body: Value,
    auth: Option<&RuntimeRequestAuth>,
) -> Value {
    let id = body.get("id").cloned().unwrap_or(Value::Null);
    let Some(method) = body.get("method").and_then(Value::as_str) else {
        return json_rpc_error(id, -32600, "missing JSON-RPC method");
    };

    let result = match method {
        "initialize" => initialize(runtime, server_id),
        "tools/list" => list_tools(runtime, server_id),
        "tools/call" => call_tool(runtime, server_id, &body, auth).await,
        "resources/list" => list_resources(runtime, server_id),
        "resources/templates/list" => list_resource_templates(runtime, server_id),
        "resources/read" => read_resource(runtime, server_id, &body, auth).await,
        "prompts/list" => list_prompts(runtime, server_id),
        "prompts/get" => get_prompt(runtime, server_id, &body, auth).await,
        _ => Err(JsonRpcDispatchError::MethodNotFound(format!(
            "method not found: {method}"
        ))),
    };

    match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(JsonRpcDispatchError::InvalidParams(message)) => json_rpc_error(id, -32602, &message),
        Err(JsonRpcDispatchError::MethodNotFound(message)) => json_rpc_error(id, -32601, &message),
        Err(JsonRpcDispatchError::Runtime(error)) => json_rpc_error(id, -32000, &error.to_string()),
    }
}

fn initialize(
    runtime: &McpRuntime,
    server_id: &McpServerId,
) -> Result<Value, JsonRpcDispatchError> {
    let snapshot = snapshot(runtime, server_id)?;
    let mut capabilities = Map::new();
    if !snapshot.tools.is_empty() {
        capabilities.insert("tools".to_string(), json!({}));
    }
    if !snapshot.resources.is_empty() || !snapshot.resource_templates.is_empty() {
        capabilities.insert("resources".to_string(), json!({}));
    }
    if !snapshot.prompts.is_empty() {
        capabilities.insert("prompts".to_string(), json!({}));
    }

    Ok(json!({
        "protocolVersion": "2025-11-25",
        "capabilities": capabilities,
        "serverInfo": {
            "name": "agentcast-gateway",
            "title": "AgentCast Gateway",
            "version": env!("CARGO_PKG_VERSION")
        }
    }))
}

fn list_tools(
    runtime: &McpRuntime,
    server_id: &McpServerId,
) -> Result<Value, JsonRpcDispatchError> {
    let snapshot = snapshot(runtime, server_id)?;
    let tools = snapshot
        .tools
        .iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "title": tool.title,
                "description": tool.description,
                "inputSchema": tool.input_schema,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({ "tools": tools }))
}

async fn call_tool(
    runtime: &McpRuntime,
    server_id: &McpServerId,
    body: &Value,
    auth: Option<&RuntimeRequestAuth>,
) -> Result<Value, JsonRpcDispatchError> {
    let params = params(body)?;
    let name = required_string(params, "name")?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let response = runtime
        .call_tool(ToolCallRequest {
            server_id: server_id.clone(),
            tool_id: McpToolId::new(name),
            arguments,
            auth: auth.cloned(),
        })
        .await?;
    Ok(response.output)
}

fn list_resources(
    runtime: &McpRuntime,
    server_id: &McpServerId,
) -> Result<Value, JsonRpcDispatchError> {
    let snapshot = snapshot(runtime, server_id)?;
    let resources = snapshot
        .resources
        .iter()
        .map(|resource| {
            json!({
                "uri": resource.uri,
                "name": resource.name,
                "title": resource.title,
                "description": resource.description,
                "mimeType": resource.mime_type,
                "size": resource.size,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({ "resources": resources }))
}

fn list_resource_templates(
    runtime: &McpRuntime,
    server_id: &McpServerId,
) -> Result<Value, JsonRpcDispatchError> {
    let snapshot = snapshot(runtime, server_id)?;
    let resource_templates = snapshot
        .resource_templates
        .iter()
        .map(|template| {
            json!({
                "uriTemplate": template.uri_template,
                "name": template.name,
                "title": template.title,
                "description": template.description,
                "mimeType": template.mime_type,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({ "resourceTemplates": resource_templates }))
}

async fn read_resource(
    runtime: &McpRuntime,
    server_id: &McpServerId,
    body: &Value,
    auth: Option<&RuntimeRequestAuth>,
) -> Result<Value, JsonRpcDispatchError> {
    let uri = required_string(params(body)?, "uri")?;
    let response = match auth {
        Some(auth) => {
            runtime
                .read_resource_with_bearer(server_id, uri, auth.bearer_token.clone())
                .await?
        }
        None => runtime.read_resource(server_id, uri).await?,
    };
    Ok(serde_json::to_value(response).unwrap_or(Value::Null))
}

fn list_prompts(
    runtime: &McpRuntime,
    server_id: &McpServerId,
) -> Result<Value, JsonRpcDispatchError> {
    let snapshot = snapshot(runtime, server_id)?;
    let prompts = snapshot
        .prompts
        .iter()
        .map(|prompt| {
            json!({
                "name": prompt.name,
                "title": prompt.title,
                "description": prompt.description,
                "arguments": prompt.arguments,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({ "prompts": prompts }))
}

async fn get_prompt(
    runtime: &McpRuntime,
    server_id: &McpServerId,
    body: &Value,
    auth: Option<&RuntimeRequestAuth>,
) -> Result<Value, JsonRpcDispatchError> {
    let params = params(body)?;
    let name = required_string(params, "name")?;
    let arguments = params
        .get("arguments")
        .and_then(|value| value.as_object())
        .cloned();
    match auth {
        Some(auth) => runtime
            .get_prompt_with_bearer(server_id, name, arguments, auth.bearer_token.clone())
            .await
            .map_err(Into::into),
        None => runtime
            .get_prompt(server_id, name, arguments)
            .await
            .map_err(Into::into),
    }
}

fn snapshot(
    runtime: &McpRuntime,
    server_id: &McpServerId,
) -> Result<RuntimeCatalogSnapshot, JsonRpcDispatchError> {
    runtime
        .snapshots()
        .into_iter()
        .find(|snapshot| snapshot.server_id == *server_id)
        .ok_or_else(|| RuntimeError::UnknownServer(server_id.to_string()).into())
}

fn params(body: &Value) -> Result<&Map<String, Value>, JsonRpcDispatchError> {
    body.get("params")
        .and_then(Value::as_object)
        .ok_or_else(|| JsonRpcDispatchError::InvalidParams("missing object params".to_string()))
}

fn required_string<'a>(
    params: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a str, JsonRpcDispatchError> {
    params
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| JsonRpcDispatchError::InvalidParams(format!("missing string params.{name}")))
}

fn json_rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message,
        },
    })
}

#[derive(Debug)]
enum JsonRpcDispatchError {
    InvalidParams(String),
    MethodNotFound(String),
    Runtime(RuntimeError),
}

impl From<RuntimeError> for JsonRpcDispatchError {
    fn from(error: RuntimeError) -> Self {
        Self::Runtime(error)
    }
}
