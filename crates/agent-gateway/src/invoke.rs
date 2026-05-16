use crate::{ActionRoute, GatewayCatalog, GatewayError, GatewayExposurePolicy, GatewayRouter};
use agent_auth::OAuthCredential;
use agent_protocol::{LauncherActionKind, ToolInvocation, ToolInvocationResult};
use agent_runtime::{McpRuntime, RuntimeCatalogSnapshot, RuntimeRequestAuth, ToolCallRequest};
use std::collections::BTreeMap;

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

    pub fn health_from_snapshots(snapshots: &[RuntimeCatalogSnapshot]) -> GatewayHealthSummary {
        let catalog = GatewayCatalog::from_snapshots(snapshots.to_vec());
        GatewayHealthSummary::from_catalog_and_snapshots(&catalog, snapshots)
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GatewayHealthSummary {
    pub server_count: usize,
    pub action_count: usize,
    pub collision_count: usize,
    pub status_counts: BTreeMap<String, usize>,
}

impl GatewayHealthSummary {
    pub fn from_catalog_and_snapshots(
        catalog: &GatewayCatalog,
        snapshots: &[RuntimeCatalogSnapshot],
    ) -> Self {
        let mut status_counts = BTreeMap::new();
        for snapshot in snapshots {
            *status_counts
                .entry(format!("{:?}", snapshot.status).to_ascii_lowercase())
                .or_insert(0) += 1;
        }
        Self {
            server_count: snapshots.len(),
            action_count: catalog.actions.len(),
            collision_count: catalog.collisions.len(),
            status_counts,
        }
    }

    pub fn status_count(&self, status: &str) -> usize {
        self.status_counts.get(status).copied().unwrap_or(0)
    }
}
