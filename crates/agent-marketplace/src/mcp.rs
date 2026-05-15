use agent_config::{
    AgentConfig, EnvMerge, McpTransport, McpUpstreamConfig, StdioUpstreamConfig,
    StreamableHttpUpstreamConfig, add_or_replace_upstream,
};
use agent_registry::{NormalizedMcpPackage, NormalizedMcpServer};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    InstallPlan, InstallStep, InstallStepKind, MarketplaceError, MarketplaceResult,
    validate_env_name, validate_env_value, validate_registry_url, validate_runtime_hint,
    validate_stdio_argv,
};

#[cfg(test)]
mod tests;

pub fn plan_mcp_server_install(server: &NormalizedMcpServer) -> MarketplaceResult<InstallPlan> {
    let package = server
        .packages
        .iter()
        .find(|package| package.transport.as_deref().unwrap_or("stdio") == "stdio")
        .or_else(|| server.packages.first())
        .ok_or_else(|| {
            MarketplaceError::InvalidTarget("server has no installable packages".into())
        })?;

    let upstream_id = upstream_id(&server.name)?;
    let mut plan = InstallPlan::new(&server.name);

    if package.registry_type == "mcpb" {
        return Err(MarketplaceError::InvalidTarget(
            "mcpb packages require artifact integrity verification before v0 install planning"
                .into(),
        ));
    }

    if package
        .transport
        .as_deref()
        .is_some_and(|transport| transport != "stdio")
    {
        return plan_remote_mcp_server_install(server, package, upstream_id, plan);
    }

    let command = package.runtime_hint.as_deref().ok_or_else(|| {
        MarketplaceError::InvalidTarget("stdio package has no runtime hint".into())
    })?;
    validate_runtime_hint(command)?;

    let args = stdio_args(package)?;
    validate_stdio_argv(command, &args)?;

    plan = plan.step(InstallStep {
        kind: InstallStepKind::VerifyRuntime,
        description: format!("Verify `{command}` is available before install"),
        target: format!("runtime.{command}"),
        preview: serde_json::json!({ "command": command }),
    });

    let env_names = package
        .environment_variables
        .iter()
        .map(|env| {
            validate_env_name(&env.name)?;
            Ok(env.name.clone())
        })
        .collect::<MarketplaceResult<Vec<_>>>()?;

    for env in &package.environment_variables {
        plan = plan.step(InstallStep {
            kind: InstallStepKind::SetEnvVar,
            description: format!("Prepare environment variable `{}`", env.name),
            target: format!("env.{}", env.name),
            preview: serde_json::json!({
                "name": env.name,
                "required": env.is_required,
                "secret": env.is_secret,
                "has_default": env.default.is_some(),
            }),
        });
    }

    plan = plan.step(InstallStep {
        kind: InstallStepKind::AddMcpUpstream,
        description: format!("Add {upstream_id} MCP upstream"),
        target: format!("mcp.upstreams.{upstream_id}"),
        preview: serde_json::json!({
            "id": upstream_id,
            "transport": "stdio",
            "command": command,
            "args": args,
            "env": env_names,
        }),
    });

    Ok(plan)
}

fn plan_remote_mcp_server_install(
    server: &NormalizedMcpServer,
    package: &NormalizedMcpPackage,
    upstream_id: String,
    mut plan: InstallPlan,
) -> MarketplaceResult<InstallPlan> {
    let remote = server
        .remotes
        .iter()
        .find(|remote| matches!(remote.transport_type.as_str(), "http" | "streamable_http"))
        .ok_or_else(|| {
            MarketplaceError::InvalidTarget(
                "remote MCP package does not include a supported HTTP endpoint".into(),
            )
        })?;
    let url = remote.url.as_deref().ok_or_else(|| {
        MarketplaceError::InvalidTarget("remote MCP endpoint is missing URL".into())
    })?;
    validate_registry_url(url)?;

    let bearer_token_env = package
        .environment_variables
        .iter()
        .find(|env| env.is_secret)
        .map(|env| {
            validate_env_name(&env.name)?;
            Ok(env.name.clone())
        })
        .transpose()?;

    for env in &package.environment_variables {
        validate_env_name(&env.name)?;
        plan = plan.step(InstallStep {
            kind: InstallStepKind::SetEnvVar,
            description: format!("Prepare environment variable `{}`", env.name),
            target: format!("env.{}", env.name),
            preview: serde_json::json!({
                "name": env.name,
                "required": env.is_required,
                "secret": env.is_secret,
                "has_default": env.default.is_some(),
            }),
        });
    }

    plan = plan.step(InstallStep {
        kind: InstallStepKind::AddMcpUpstream,
        description: format!("Add {upstream_id} remote MCP upstream"),
        target: format!("mcp.upstreams.{upstream_id}"),
        preview: serde_json::json!({
            "id": upstream_id,
            "transport": "streamable_http",
            "url": url,
            "bearer_token_env": bearer_token_env,
        }),
    });

    Ok(plan)
}

#[derive(Debug, Clone, Default, Eq, PartialEq, serde::Serialize)]
pub struct ApplyInstallPlanResult {
    pub added_or_replaced_upstreams: Vec<String>,
    pub env_keys_required: Vec<String>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct InstallEnvResolution {
    pub values: BTreeMap<String, String>,
    pub missing_required: Vec<String>,
}

pub fn apply_install_plan_to_config(
    config: &mut AgentConfig,
    plan: &InstallPlan,
) -> MarketplaceResult<ApplyInstallPlanResult> {
    let mut result = ApplyInstallPlanResult::default();
    for step in &plan.steps {
        match step.kind {
            InstallStepKind::VerifyRuntime => {}
            InstallStepKind::SetEnvVar => {
                if let Some(name) = step.preview.get("name").and_then(Value::as_str) {
                    result.env_keys_required.push(name.to_string());
                }
            }
            InstallStepKind::AddMcpUpstream => {
                let id = step
                    .preview
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        MarketplaceError::InvalidTarget(
                            "MCP upstream install step is missing preview.id".into(),
                        )
                    })?;
                let command = step
                    .preview
                    .get("command")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let transport =
                    match step.preview.get("transport").and_then(Value::as_str) {
                        Some("streamable_http") => {
                            let url = step.preview.get("url").and_then(Value::as_str).ok_or_else(
                                || {
                                    MarketplaceError::InvalidTarget(
                                        "MCP upstream install step is missing preview.url".into(),
                                    )
                                },
                            )?;
                            McpTransport::StreamableHttp(StreamableHttpUpstreamConfig {
                                url: url.to_string(),
                                bearer_token_env: step
                                    .preview
                                    .get("bearer_token_env")
                                    .and_then(Value::as_str)
                                    .map(str::to_string),
                            })
                        }
                        _ => {
                            let command = command.ok_or_else(|| {
                                MarketplaceError::InvalidTarget(
                                    "MCP upstream install step is missing preview.command".into(),
                                )
                            })?;
                            let args = step
                                .preview
                                .get("args")
                                .and_then(Value::as_array)
                                .ok_or_else(|| {
                                    MarketplaceError::InvalidTarget(
                                        "MCP upstream install step is missing preview.args".into(),
                                    )
                                })?
                                .iter()
                                .map(|value| {
                                    value.as_str().map(str::to_string).ok_or_else(|| {
                                        MarketplaceError::InvalidTarget(
                                            "MCP upstream args must be strings".into(),
                                        )
                                    })
                                })
                                .collect::<MarketplaceResult<Vec<_>>>()?;
                            McpTransport::Stdio(StdioUpstreamConfig {
                                command,
                                args,
                                cwd: None,
                                env: Default::default(),
                            })
                        }
                    };

                let upstream = McpUpstreamConfig::new(id, transport)
                    .map_err(|error| MarketplaceError::InvalidTarget(error.to_string()))?;
                add_or_replace_upstream(config, upstream)
                    .map_err(|error| MarketplaceError::InvalidTarget(error.to_string()))?;
                result.added_or_replaced_upstreams.push(id.to_string());
            }
        }
    }
    Ok(result)
}

pub fn resolve_install_env(
    server: &NormalizedMcpServer,
    supplied: &BTreeMap<String, String>,
) -> MarketplaceResult<InstallEnvResolution> {
    let mut allowed = BTreeSet::new();
    let mut resolution = InstallEnvResolution::default();

    for package in &server.packages {
        for env in &package.environment_variables {
            validate_env_name(&env.name)?;
            allowed.insert(env.name.clone());
            let value = supplied
                .get(&env.name)
                .cloned()
                .or_else(|| env.default.clone());
            match value {
                Some(value) => {
                    validate_env_value(&value)?;
                    resolution.values.insert(env.name.clone(), value);
                }
                None if env.is_required => resolution.missing_required.push(env.name.clone()),
                None => {}
            }
        }
    }

    for key in supplied.keys() {
        if !allowed.contains(key) {
            return Err(MarketplaceError::InvalidTarget(format!(
                "env value `{key}` is not declared by the registry package"
            )));
        }
    }

    Ok(resolution)
}

pub fn install_env_merge(resolution: &InstallEnvResolution) -> MarketplaceResult<EnvMerge> {
    if !resolution.missing_required.is_empty() {
        return Err(MarketplaceError::InvalidTarget(format!(
            "missing required env values: {}",
            resolution.missing_required.join(", ")
        )));
    }
    Ok(EnvMerge::new(resolution.values.clone()))
}

fn stdio_args(package: &NormalizedMcpPackage) -> MarketplaceResult<Vec<String>> {
    let mut args = values_to_args("runtime_arguments", &package.runtime_arguments)?;
    args.push(package.identifier.clone());
    args.extend(values_to_args(
        "package_arguments",
        &package.package_arguments,
    )?);
    Ok(args)
}

fn values_to_args(field: &str, values: &[Value]) -> MarketplaceResult<Vec<String>> {
    values
        .iter()
        .map(|value| match value {
            Value::String(value) => Ok(value.clone()),
            Value::Number(value) => Ok(value.to_string()),
            Value::Bool(value) => Ok(value.to_string()),
            _ => Err(MarketplaceError::InvalidTarget(format!(
                "{field} entries must be scalar values"
            ))),
        })
        .collect()
}

fn upstream_id(name: &str) -> MarketplaceResult<String> {
    let raw = name.rsplit('/').next().unwrap_or(name).trim();
    if raw.is_empty() {
        return Err(MarketplaceError::InvalidTarget(
            "server name cannot produce upstream id".into(),
        ));
    }
    let id = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if id.is_empty() {
        return Err(MarketplaceError::InvalidTarget(
            "server name cannot produce upstream id".into(),
        ));
    }
    Ok(id)
}
