use crate::ConfigError;
use agent_protocol::{McpServerConfig, McpServerId, McpTransportConfig};
use serde_json::Value;
use std::collections::BTreeMap;

#[cfg(test)]
mod tests;

pub fn parse_mcp_json(raw: &str) -> Result<Vec<McpServerConfig>, ConfigError> {
    let stripped = strip_jsonc_comments(raw);
    let value: Value = serde_json::from_str(&stripped)?;
    let object = value.as_object().cloned().unwrap_or_default();
    let servers = object
        .get("mcpServers")
        .or_else(|| object.get("servers"))
        .or_else(|| object.get("mcp"))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let mut imported = Vec::new();
    for (name, spec) in servers {
        let server = parse_server(&name, &spec)?;
        imported.push(server);
    }
    imported.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(imported)
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

    if let Some(command) = object.get("command").and_then(Value::as_str) {
        let args = object
            .get("args")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
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
