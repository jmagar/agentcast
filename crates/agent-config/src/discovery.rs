use crate::{ConfigError, parse_mcp_json, parse_mcp_json_with_options};
use agent_protocol::McpServerConfig;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredMcpServer {
    pub config: McpServerConfig,
    pub source_client: String,
    pub source_path: PathBuf,
    pub env_key_count: usize,
}

pub fn discover_known_mcp_configs(home: &Path) -> Vec<DiscoveredMcpServer> {
    let mut discovered = Vec::new();
    discovered.extend(discover_claude_code(home));
    discovered.extend(discover_claude_desktop(home));
    discovered.extend(discover_codex(home));
    discovered.extend(discover_gemini(home));
    discovered.extend(discover_cursor(home));
    discovered.extend(discover_vscode(home));
    discovered.extend(discover_windsurf(home));
    discovered.extend(discover_opencode(home));
    dedupe_first_seen(discovered)
}

fn config_roots(home: &Path) -> Vec<PathBuf> {
    let mut roots = vec![home.join(".config")];
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg);
        if path != roots[0] {
            roots.push(path);
        }
    }
    roots
}

fn discover_claude_code(home: &Path) -> Vec<DiscoveredMcpServer> {
    let strict_paths = [
        home.join(".claude").join("settings.local.json"),
        home.join(".claude").join("settings.json"),
    ];
    let legacy_paths = [
        home.join(".claude").join("mcp.json"),
        home.join(".claude.json"),
    ];

    let mut discovered = Vec::new();
    for path in strict_paths {
        discovered.extend(discover_json_path(&path, "claude-code", false));
    }
    for path in legacy_paths {
        discovered.extend(discover_json_path(&path, "claude-code", true));
    }
    discovered
}

fn discover_claude_desktop(home: &Path) -> Vec<DiscoveredMcpServer> {
    let mut paths = config_roots(home)
        .into_iter()
        .map(|root| root.join("Claude").join("claude_desktop_config.json"))
        .collect::<Vec<_>>();
    paths.extend([
        home.join("Library")
            .join("Application Support")
            .join("Claude")
            .join("claude_desktop_config.json"),
        home.join("AppData")
            .join("Roaming")
            .join("Claude")
            .join("claude_desktop_config.json"),
    ]);
    paths
        .into_iter()
        .flat_map(|path| discover_json_path(&path, "claude-desktop", false))
        .collect()
}

fn discover_codex(home: &Path) -> Vec<DiscoveredMcpServer> {
    let path = home.join(".codex").join("config.toml");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(value) = toml::from_str::<toml::Value>(&raw) else {
        return Vec::new();
    };
    let Some(servers) = value.get("mcp_servers").and_then(toml::Value::as_table) else {
        return Vec::new();
    };

    let mut root = BTreeMap::new();
    for (name, value) in servers {
        if let Ok(value) = serde_json::to_value(value) {
            root.insert(name.clone(), value);
        }
    }
    discover_json_value(
        &Value::Object(root.into_iter().collect()),
        &path,
        "codex",
        true,
    )
}

fn discover_gemini(home: &Path) -> Vec<DiscoveredMcpServer> {
    [
        home.join(".gemini").join("mcp.json"),
        home.join(".gemini").join("settings.json"),
    ]
    .into_iter()
    .flat_map(|path| discover_json_path(&path, "gemini", false))
    .collect()
}

fn discover_cursor(home: &Path) -> Vec<DiscoveredMcpServer> {
    [
        home.join(".cursor").join("mcp.json"),
        home.join(".cursor").join("mcp_config.json"),
    ]
    .into_iter()
    .flat_map(|path| discover_json_path(&path, "cursor", false))
    .collect()
}

fn discover_vscode(home: &Path) -> Vec<DiscoveredMcpServer> {
    let mut paths = Vec::new();
    for root in config_roots(home) {
        paths.push(root.join("Code").join("User").join("mcp.json"));
        paths.push(root.join("Code - Insiders").join("User").join("mcp.json"));
        paths.push(root.join("Antigravity").join("User").join("mcp.json"));
    }
    paths.push(home.join(".vscode").join("mcp.json"));
    paths
        .into_iter()
        .flat_map(|path| discover_json_path(&path, "vscode", false))
        .collect()
}

fn discover_windsurf(home: &Path) -> Vec<DiscoveredMcpServer> {
    [
        home.join(".windsurf").join("mcp.json"),
        home.join(".codeium")
            .join("windsurf")
            .join("mcp_config.json"),
    ]
    .into_iter()
    .flat_map(|path| discover_json_path(&path, "windsurf", false))
    .collect()
}

fn discover_opencode(home: &Path) -> Vec<DiscoveredMcpServer> {
    [home.join(".opencode").join("mcp.json")]
        .into_iter()
        .flat_map(|path| discover_json_path(&path, "opencode", false))
        .collect()
}

fn discover_json_path(
    path: &Path,
    source_client: &str,
    allow_root_fallback: bool,
) -> Vec<DiscoveredMcpServer> {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    discover_json_raw(&raw, path, source_client, allow_root_fallback).unwrap_or_default()
}

fn discover_json_raw(
    raw: &str,
    path: &Path,
    source_client: &str,
    allow_root_fallback: bool,
) -> Result<Vec<DiscoveredMcpServer>, ConfigError> {
    let configs = if allow_root_fallback {
        parse_mcp_json_with_options(raw, true)?
    } else {
        parse_mcp_json(raw)?
    };
    let stripped = crate::mcp_json::strip_jsonc_comments(raw);
    let value = serde_json::from_str::<Value>(&stripped).unwrap_or(Value::Null);
    Ok(configs
        .into_iter()
        .map(|mut config| {
            crate::mcp_json::scrub_env_values(&mut config);
            let env_key_count = value
                .get("mcpServers")
                .or_else(|| value.get("servers"))
                .or_else(|| value.get("mcp"))
                .and_then(Value::as_object)
                .and_then(|servers| servers.get(config.name.as_str()))
                .or_else(|| value.get(config.name.as_str()))
                .and_then(|entry| entry.get("env"))
                .and_then(Value::as_object)
                .map_or(0, serde_json::Map::len);
            DiscoveredMcpServer {
                config,
                source_client: source_client.to_string(),
                source_path: path.to_path_buf(),
                env_key_count,
            }
        })
        .collect())
}

fn discover_json_value(
    value: &Value,
    path: &Path,
    source_client: &str,
    allow_root_fallback: bool,
) -> Vec<DiscoveredMcpServer> {
    discover_json_raw(
        &serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string()),
        path,
        source_client,
        allow_root_fallback,
    )
    .unwrap_or_default()
}

fn dedupe_first_seen(servers: Vec<DiscoveredMcpServer>) -> Vec<DiscoveredMcpServer> {
    let mut seen = BTreeSet::new();
    servers
        .into_iter()
        .filter(|server| seen.insert(server.config.name.clone()))
        .collect()
}
