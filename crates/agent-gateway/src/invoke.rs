use crate::{ActionRoute, GatewayCatalog, GatewayError, GatewayExposurePolicy, GatewayRouter};
use agent_auth::OAuthCredential;
use agent_protocol::{LauncherActionKind, ToolInvocation, ToolInvocationResult};
use agent_runtime::{McpRuntime, RuntimeCatalogSnapshot, RuntimeRequestAuth, ToolCallRequest};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct GatewayService {
    pub catalog: GatewayCatalog,
    router: GatewayRouter,
}

impl GatewayService {
    pub fn from_runtime_snapshots(snapshots: Vec<RuntimeCatalogSnapshot>) -> Self {
        Self::from_runtime_snapshots_with_policy(snapshots, &GatewayExposurePolicy::default())
    }

    pub fn from_runtime_snapshots_with_policy(
        snapshots: Vec<RuntimeCatalogSnapshot>,
        exposure_policy: &GatewayExposurePolicy,
    ) -> Self {
        let catalog = GatewayCatalog::from_snapshots_with_policy(snapshots, exposure_policy);
        let routes = catalog
            .actions
            .iter()
            .map(|action| match &action.kind {
                LauncherActionKind::McpTool { server_id, tool_id } => ActionRoute {
                    action_id: action.id.clone(),
                    server_id: server_id.clone(),
                    tool_id: tool_id.clone(),
                },
            })
            .collect();
        Self {
            catalog,
            router: GatewayRouter::new(routes),
        }
    }

    pub fn router(&self) -> &GatewayRouter {
        &self.router
    }

    pub async fn invoke(
        &self,
        runtime: &McpRuntime,
        invocation: ToolInvocation,
    ) -> Result<ToolInvocationResult, GatewayError> {
        let route = self
            .router
            .resolve(&invocation.action_id)
            .ok_or_else(|| GatewayError::UnknownAction(invocation.action_id.to_string()))?;
        let response = runtime
            .call_tool(ToolCallRequest {
                server_id: route.server_id.clone(),
                tool_id: route.tool_id.clone(),
                arguments: invocation.arguments,
                auth: None,
            })
            .await
            .map_err(|error| GatewayError::Runtime(error.to_string()))?;
        Ok(ToolInvocationResult {
            action_id: invocation.action_id,
            output: response.output,
        })
    }

    pub async fn invoke_with_credential(
        &self,
        runtime: &McpRuntime,
        invocation: ToolInvocation,
        credential: Option<&OAuthCredential>,
    ) -> Result<ToolInvocationResult, GatewayError> {
        let route = self
            .router
            .resolve(&invocation.action_id)
            .ok_or_else(|| GatewayError::UnknownAction(invocation.action_id.to_string()))?;
        let response = runtime
            .call_tool(ToolCallRequest {
                server_id: route.server_id.clone(),
                tool_id: route.tool_id.clone(),
                arguments: invocation.arguments,
                auth: credential
                    .map(|credential| RuntimeRequestAuth::bearer(credential.access_token.clone())),
            })
            .await
            .map_err(|error| GatewayError::Runtime(error.to_string()))?;
        Ok(ToolInvocationResult {
            action_id: invocation.action_id,
            output: response.output,
        })
    }
}
