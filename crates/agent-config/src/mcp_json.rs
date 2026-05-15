use crate::ConfigError;
use agent_protocol::{McpServerConfig, McpServerId, McpTransportConfig};
use serde_json::Value;
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

pub fn parse_mcp_json(raw: &str) -> Result<Vec<McpServerConfig>, ConfigError> {
    parse_mcp_json_with_options(raw, false)
}

pub fn parse_mcp_json_with_options(
    raw: &str,
    allow_root_fallback: bool,
) -> Result<Vec<McpServerConfig>, ConfigError> {
    let stripped = strip_jsonc_comments(raw);
    let value: Value = serde_json::from_str(&stripped)?;
    let object = value.as_object().cloned().unwrap_or_default();
    let servers = mcp_servers(&value, &object, allow_root_fallback);

    let mut imported = Vec::new();
    for (name, spec) in servers {
        let server = parse_server(&name, spec)?;
        imported.push(server);
    }
    imported.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(imported)
}

pub(crate) fn scrub_env_values(config: &mut McpServerConfig) {
    if let McpTransportConfig::Stdio { env, .. } = &mut config.transport {
        env.clear();
    }
}

fn mcp_servers<'a>(
    value: &'a Value,
    object: &'a serde_json::Map<String, Value>,
    allow_root_fallback: bool,
) -> Vec<(String, &'a Value)> {
    for key in ["mcpServers", "servers", "mcp"] {
        if let Some(servers) = object.get(key).and_then(Value::as_object) {
            return servers
                .iter()
                .map(|(name, spec)| (name.clone(), spec))
                .collect();
        }
    }

    if allow_root_fallback && root_looks_like_servers(value) {
        return object
            .iter()
            .map(|(name, spec)| (name.clone(), spec))
            .collect();
    }

    Vec::new()
}

fn root_looks_like_servers(value: &Value) -> bool {
    value
        .as_object()
        .map(|object| {
            object.values().any(|entry| {
                entry
                    .as_object()
                    .map(|entry| entry.contains_key("command") || entry.contains_key("url"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn parse_server(name: &str, spec: &Value) -> Result<McpServerConfig, ConfigError> {
    let object = spec.as_object().cloned().unwrap_or_default();
    let env = object
        .get("env")
        .and_then(Value::as_object)
        .map(|values| {
            values
                .iter()
                .map(|(key, value)| (key.clone(), value.as_str().unwrap_or_default().to_string()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let env_keys = env.keys().cloned().collect::<Vec<_>>();

    if let Some((command, args)) = command_and_args(&object) {
        return Ok(McpServerConfig {
            id: McpServerId::new(name),
            name: name.to_string(),
            enabled: false,
            transport: McpTransportConfig::Stdio {
                command: command.to_string(),
                args,
                env,
            },
            env_keys,
        });
    }

    let url = ["url", "baseUrl", "base_url", "serverUrl", "server_url"]
        .iter()
        .find_map(|key| object.get(*key).and_then(Value::as_str));

    if let Some(url) = url {
        return Ok(McpServerConfig {
            id: McpServerId::new(name),
            name: name.to_string(),
            enabled: false,
            transport: McpTransportConfig::StreamableHttp {
                url: url.to_string(),
                bearer_token_env: object
                    .get("bearer_token_env")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
            },
            env_keys,
        });
    }

    Err(ConfigError::MissingTarget {
        name: name.to_string(),
    })
}

fn command_and_args(object: &serde_json::Map<String, Value>) -> Option<(String, Vec<String>)> {
    match object.get("command") {
        Some(Value::String(command)) => Some((command.clone(), array_strings(object.get("args")))),
        Some(Value::Array(command)) => {
            let first = command.first()?.as_str()?.to_string();
            let args = if command.len() > 1 {
                command[1..]
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect()
            } else {
                array_strings(object.get("args"))
            };
            Some((first, args))
        }
        _ => None,
    }
}

fn array_strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn strip_jsonc_comments(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
            output.push(ch);
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'/') {
            chars.next();
            for next in chars.by_ref() {
                if next == '\n' {
                    output.push('\n');
                    break;
                }
            }
            continue;
        }

        output.push(ch);
    }

    output
}
