use crate::{
    McpError, McpPromptMetadata, McpReadResourceResult, McpResourceMetadata,
    McpResourceTemplateMetadata, McpResult, McpToolMetadata, McpToolResult, StdioConnection,
};
use rmcp::model::{CallToolRequestParams, GetPromptRequestParams, ReadResourceRequestParams};
use rmcp::service::RunningService;
use rmcp::transport::{
    StreamableHttpClientTransport, TokioChildProcess,
    streamable_http_client::StreamableHttpClientTransportConfig,
};
use rmcp::{RoleClient, ServiceExt};
use serde_json::{Map, Value};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
pub struct McpClientOptions {
    pub cancellation_token: CancellationToken,
}

impl Default for McpClientOptions {
    fn default() -> Self {
        Self {
            cancellation_token: CancellationToken::new(),
        }
    }
}

#[derive(Debug)]
pub struct McpClient {
    service: RunningService<RoleClient, ()>,
}

impl McpClient {
    pub async fn connect_stdio(
        connection: StdioConnection,
        options: McpClientOptions,
    ) -> McpResult<Self> {
        let transport = TokioChildProcess::builder(connection.command())
            .spawn()
            .map_err(|error| McpError::Connection(error.to_string()))?
            .0;
        let service = ()
            .serve_with_ct(transport, options.cancellation_token)
            .await
            .map_err(|error| McpError::Connection(error.to_string()))?;

        Ok(Self { service })
    }

    pub async fn connect_streamable_http(
        url: impl Into<String>,
        bearer_token: Option<String>,
        options: McpClientOptions,
    ) -> McpResult<Self> {
        let mut config = StreamableHttpClientTransportConfig::with_uri(url.into());
        if let Some(token) = bearer_token {
            config = config.auth_header(token);
        }
        let transport = StreamableHttpClientTransport::from_config(config);
        let service = ()
            .serve_with_ct(transport, options.cancellation_token)
            .await
            .map_err(|error| McpError::Connection(error.to_string()))?;

        Ok(Self { service })
    }

    pub async fn list_tools(&self) -> McpResult<Vec<McpToolMetadata>> {
        self.service
            .peer()
            .list_all_tools()
            .await
            .map_err(|error| McpError::Protocol(error.to_string()))
            .map(|tools| tools.into_iter().map(Into::into).collect())
    }

    pub async fn call_tool(&self, name: &str, arguments: Value) -> McpResult<McpToolResult> {
        let arguments = match arguments {
            Value::Object(arguments) => arguments,
            Value::Null => Map::new(),
            _ => return Err(McpError::InvalidArguments),
        };
        let result = self
            .service
            .peer()
            .call_tool(CallToolRequestParams::new(name.to_string()).with_arguments(arguments))
            .await
            .map_err(|error| McpError::Tool(error.to_string()))?;
        Ok(result.into())
    }

    pub async fn list_resources(&self) -> McpResult<Vec<McpResourceMetadata>> {
        self.service
            .peer()
            .list_all_resources()
            .await
            .map_err(|error| McpError::Protocol(error.to_string()))
            .map(|resources| resources.into_iter().map(Into::into).collect())
    }

    pub async fn list_resource_templates(&self) -> McpResult<Vec<McpResourceTemplateMetadata>> {
        let result = self
            .service
            .peer()
            .list_resource_templates(None)
            .await
            .map_err(|error| McpError::Protocol(error.to_string()))?;
        Ok(result
            .resource_templates
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub async fn read_resource(&self, uri: &str) -> McpResult<McpReadResourceResult> {
        self.service
            .peer()
            .read_resource(ReadResourceRequestParams::new(uri.to_string()))
            .await
            .map_err(|error| McpError::Protocol(error.to_string()))
            .map(Into::into)
    }

    pub async fn list_prompts(&self) -> McpResult<Vec<McpPromptMetadata>> {
        self.service
            .peer()
            .list_all_prompts()
            .await
            .map_err(|error| McpError::Protocol(error.to_string()))
            .map(|prompts| prompts.into_iter().map(Into::into).collect())
    }

    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<Map<String, Value>>,
    ) -> McpResult<Value> {
        self.service
            .peer()
            .get_prompt(match arguments {
                Some(arguments) => {
                    GetPromptRequestParams::new(name.to_string()).with_arguments(arguments)
                }
                None => GetPromptRequestParams::new(name.to_string()),
            })
            .await
            .map_err(|error| McpError::Protocol(error.to_string()))
            .map(|result| serde_json::to_value(result).unwrap_or(Value::Null))
    }
}
