use async_trait::async_trait;
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResult, JsonObject, ListToolsResult, PaginatedRequestParams,
        ServerCapabilities, ServerInfo, Tool,
    },
    service::{MaybeSendFuture, RequestContext, RoleServer},
    transport,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{error::Error, sync::Arc};

const LIST_ACTIONS_TOOL: &str = "agentcast_gateway_list_actions";
const SEARCH_ACTIONS_TOOL: &str = "agentcast_gateway_search_actions";
const CALL_ACTION_TOOL: &str = "agentcast_gateway_call_action";
const LIST_SERVERS_TOOL: &str = "agentcast_gateway_list_servers";
const LIST_RESOURCES_TOOL: &str = "agentcast_gateway_list_resources";
const READ_RESOURCE_TOOL: &str = "agentcast_gateway_read_resource";
const LIST_PROMPTS_TOOL: &str = "agentcast_gateway_list_prompts";
const GET_PROMPT_TOOL: &str = "agentcast_gateway_get_prompt";
const GATEWAY_STATUS_TOOL: &str = "agentcast_gateway_status";

#[async_trait]
pub trait GatewayMcpBackend: Send + Sync + 'static {
    fn gateway_status(&self) -> GatewayMcpStatus;

    fn list_servers(&self) -> Vec<GatewayMcpServer>;

    fn list_actions(&self) -> Vec<GatewayMcpAction>;

    fn search_actions(&self, query: &str, limit: usize) -> Vec<GatewayMcpSearchResult>;

    async fn call_action(&self, action_id: &str, arguments: Value) -> Result<Value, String>;

    fn list_resources(&self, server_id: Option<&str>) -> Vec<GatewayMcpResource>;

    async fn read_resource(&self, server_id: &str, uri: &str) -> Result<Value, String>;

    fn list_prompts(&self, server_id: Option<&str>) -> Vec<GatewayMcpPrompt>;

    async fn get_prompt(
        &self,
        server_id: &str,
        name: &str,
        arguments: Option<JsonObject>,
    ) -> Result<Value, String>;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GatewayMcpStatus {
    pub server_count: usize,
    pub action_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GatewayMcpServer {
    pub id: String,
    pub name: String,
    pub status: String,
    pub tool_count: usize,
    pub resource_count: usize,
    pub prompt_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GatewayMcpAction {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GatewayMcpSearchResult {
    pub action_id: String,
    pub name: String,
    pub score: u16,
    pub match_kind: String,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GatewayMcpResource {
    pub server_id: String,
    pub uri: String,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GatewayMcpPrompt {
    pub server_id: String,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub arguments: Value,
}

#[derive(Clone)]
pub struct AgentCastMcpServer<B> {
    backend: Arc<B>,
}

impl<B> AgentCastMcpServer<B>
where
    B: GatewayMcpBackend,
{
    pub fn new(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
        }
    }

    pub fn tools() -> Vec<Tool> {
        vec![
            Tool::new(
                LIST_SERVERS_TOOL,
                "List configured AgentCast upstream MCP servers and catalog counts.",
                object_schema(vec![]),
            )
            .with_title("List gateway servers"),
            Tool::new(
                LIST_ACTIONS_TOOL,
                "List AgentCast gateway actions available from configured upstream MCP servers.",
                object_schema(vec![]),
            )
            .with_title("List gateway actions"),
            Tool::new(
                SEARCH_ACTIONS_TOOL,
                "Search AgentCast gateway actions by name, description, server, and tool metadata.",
                object_schema(vec![
                    ("q", string_schema("Search query.")),
                    ("limit", integer_schema("Maximum number of results.")),
                ]),
            )
            .with_title("Search gateway actions"),
            Tool::new(
                CALL_ACTION_TOOL,
                "Call an AgentCast gateway action by action id with JSON arguments.",
                object_schema(vec![
                    ("action_id", string_schema("Gateway action id.")),
                    ("arguments", json!({ "type": "object" })),
                ]),
            )
            .with_title("Call gateway action"),
            Tool::new(
                LIST_RESOURCES_TOOL,
                "List resources exposed by configured upstream MCP servers.",
                object_schema(vec![(
                    "server_id",
                    optional_string_schema("Optional upstream server id filter."),
                )]),
            )
            .with_title("List gateway resources"),
            Tool::new(
                READ_RESOURCE_TOOL,
                "Read a resource through the AgentCast gateway runtime.",
                object_schema(vec![
                    ("server_id", string_schema("Upstream server id.")),
                    ("uri", string_schema("Resource URI.")),
                ]),
            )
            .with_title("Read gateway resource"),
            Tool::new(
                LIST_PROMPTS_TOOL,
                "List prompts exposed by configured upstream MCP servers.",
                object_schema(vec![(
                    "server_id",
                    optional_string_schema("Optional upstream server id filter."),
                )]),
            )
            .with_title("List gateway prompts"),
            Tool::new(
                GET_PROMPT_TOOL,
                "Get a prompt through the AgentCast gateway runtime.",
                object_schema(vec![
                    ("server_id", string_schema("Upstream server id.")),
                    ("name", string_schema("Prompt name.")),
                    ("arguments", optional_object_schema("Prompt arguments.")),
                ]),
            )
            .with_title("Get gateway prompt"),
            Tool::new(
                GATEWAY_STATUS_TOOL,
                "Summarize AgentCast gateway server and action counts.",
                object_schema(vec![]),
            )
            .with_title("Gateway status"),
        ]
    }

    pub async fn call_gateway_tool(&self, request: CallToolRequestParams) -> Result<Value, String> {
        let arguments = request.arguments.unwrap_or_default();
        match request.name.as_ref() {
            LIST_SERVERS_TOOL => {
                serde_json::to_value(self.backend.list_servers()).map_err(|error| error.to_string())
            }
            LIST_ACTIONS_TOOL => {
                serde_json::to_value(self.backend.list_actions()).map_err(|error| error.to_string())
            }
            SEARCH_ACTIONS_TOOL => {
                let query = required_string(&arguments, "q")?;
                let limit = optional_usize(&arguments, "limit").unwrap_or(20);
                serde_json::to_value(self.backend.search_actions(&query, limit))
                    .map_err(|error| error.to_string())
            }
            CALL_ACTION_TOOL => {
                let action_id = required_string(&arguments, "action_id")?;
                let arguments = arguments
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                self.backend.call_action(&action_id, arguments).await
            }
            LIST_RESOURCES_TOOL => {
                let server_id = optional_string(&arguments, "server_id");
                serde_json::to_value(self.backend.list_resources(server_id.as_deref()))
                    .map_err(|error| error.to_string())
            }
            READ_RESOURCE_TOOL => {
                let server_id = required_string(&arguments, "server_id")?;
                let uri = required_string(&arguments, "uri")?;
                self.backend.read_resource(&server_id, &uri).await
            }
            LIST_PROMPTS_TOOL => {
                let server_id = optional_string(&arguments, "server_id");
                serde_json::to_value(self.backend.list_prompts(server_id.as_deref()))
                    .map_err(|error| error.to_string())
            }
            GET_PROMPT_TOOL => {
                let server_id = required_string(&arguments, "server_id")?;
                let name = required_string(&arguments, "name")?;
                let prompt_arguments = optional_object(&arguments, "arguments")?;
                self.backend
                    .get_prompt(&server_id, &name, prompt_arguments)
                    .await
            }
            GATEWAY_STATUS_TOOL => serde_json::to_value(self.backend.gateway_status())
                .map_err(|error| error.to_string()),
            other => Err(format!("unknown AgentCast MCP tool: {other}")),
        }
    }
}

impl<B> ServerHandler for AgentCastMcpServer<B>
where
    B: GatewayMcpBackend,
{
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("AgentCast gateway tools for listing, searching, and invoking configured upstream MCP actions.")
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + MaybeSendFuture + '_ {
        std::future::ready(Ok(ListToolsResult::with_all_items(Self::tools())))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        Self::tools()
            .into_iter()
            .find(|tool| tool.name.as_ref() == name)
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        match self.call_gateway_tool(request).await {
            Ok(value) => Ok(tool_result(value, false)),
            Err(message) => Ok(tool_result(json!({ "error": message }), true)),
        }
    }
}

pub async fn serve_gateway_stdio<B>(backend: B) -> Result<(), Box<dyn Error + Send + Sync>>
where
    B: GatewayMcpBackend,
{
    AgentCastMcpServer::new(backend)
        .serve(transport::stdio())
        .await?
        .waiting()
        .await?;
    Ok(())
}

fn tool_result(value: Value, is_error: bool) -> CallToolResult {
    if is_error {
        CallToolResult::structured_error(value)
    } else {
        CallToolResult::structured(value)
    }
}

fn required_string(arguments: &JsonObject, key: &str) -> Result<String, String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("missing required string argument `{key}`"))
}

fn optional_string(arguments: &JsonObject, key: &str) -> Option<String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn optional_object(arguments: &JsonObject, key: &str) -> Result<Option<JsonObject>, String> {
    match arguments.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Object(object)) => Ok(Some(object.clone())),
        Some(_) => Err(format!("optional argument `{key}` must be an object")),
    }
}

fn optional_usize(arguments: &JsonObject, key: &str) -> Option<usize> {
    arguments
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
}

fn object_schema(properties: Vec<(&str, Value)>) -> Arc<JsonObject> {
    let required = properties
        .iter()
        .filter(|(_, schema)| {
            !schema
                .get("x-agentcast-optional")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .map(|(name, _)| Value::String((*name).to_string()))
        .collect::<Vec<_>>();
    let properties = properties
        .into_iter()
        .map(|(name, mut schema)| {
            if let Some(object) = schema.as_object_mut() {
                object.remove("x-agentcast-optional");
            }
            (name.to_string(), schema)
        })
        .collect::<JsonObject>();
    schema_object(json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    }))
}

fn string_schema(description: &str) -> Value {
    json!({
        "type": "string",
        "description": description
    })
}

fn optional_string_schema(description: &str) -> Value {
    json!({
        "type": "string",
        "description": description,
        "x-agentcast-optional": true
    })
}

fn optional_object_schema(description: &str) -> Value {
    json!({
        "type": "object",
        "description": description,
        "x-agentcast-optional": true
    })
}

fn integer_schema(description: &str) -> Value {
    json!({
        "type": "integer",
        "minimum": 1,
        "description": description
    })
}

fn schema_object(value: Value) -> Arc<JsonObject> {
    Arc::new(value.as_object().cloned().unwrap_or_default())
}

#[cfg(test)]
mod tests;
