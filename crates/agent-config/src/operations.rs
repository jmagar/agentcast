use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_json::Value as JsonValue;
use toml::{Table, Value as TomlValue};

use crate::{
    AgentConfig, AgentPaths, ConfigError, ConfigResult, EnvWriteResult, get_env_value,
    list_env_keys, set_env_value, unset_env_value, value_path,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ConfigValueRecord {
    pub key: String,
    pub value: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ConfigWriteRecord {
    pub path: String,
    pub key: String,
    pub validation_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ConfigUnsetRecord {
    pub path: String,
    pub key: String,
    pub removed: bool,
    pub validation_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ConfigPathsRecord {
    pub config: String,
    pub env_file: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EnvValueRecord {
    pub key: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EnvWriteRecord {
    pub path: String,
    pub key: String,
    pub previous: Option<String>,
    pub backup_written: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EnvUnsetRecord {
    pub path: String,
    pub key: String,
    pub removed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EnvListRecord {
    pub path: String,
    pub keys: Vec<String>,
}

pub fn config_get(path: &Path, key: &str) -> ConfigResult<ConfigValueRecord> {
    let root = load_or_empty_toml(path)?;
    let value = value_path::get_value(&root, key)?.map(toml_to_json);
    Ok(ConfigValueRecord {
        key: key.to_string(),
        value,
    })
}

pub fn config_set(path: &Path, key: &str, value: &str) -> ConfigResult<ConfigWriteRecord> {
    let mut root = load_or_empty_toml(path)?;
    value_path::set_value(&mut root, key, value)?;
    ensure_parent(path)?;
    fs::write(path, toml::to_string_pretty(&root)?)?;
    Ok(ConfigWriteRecord {
        path: path.display().to_string(),
        key: key.to_string(),
        validation_error: validation_error(&root),
    })
}

pub fn config_unset(path: &Path, key: &str) -> ConfigResult<ConfigUnsetRecord> {
    let mut root = load_or_empty_toml(path)?;
    let removed = value_path::unset_value(&mut root, key)?;
    if removed {
        fs::write(path, toml::to_string_pretty(&root)?)?;
    }
    Ok(ConfigUnsetRecord {
        path: path.display().to_string(),
        key: key.to_string(),
        removed,
        validation_error: validation_error(&root),
    })
}

pub fn config_list(path: &Path) -> ConfigResult<JsonValue> {
    let root = load_or_empty_toml(path)?;
    Ok(toml_to_json(root))
}

pub fn config_validate(path: &Path) -> ConfigResult<AgentConfig> {
    let root = load_or_empty_toml(path)?;
    value_path::validate_as_agent_config(&root)
}

pub fn config_paths(config: &Path, env_file: &Path) -> ConfigPathsRecord {
    ConfigPathsRecord {
        config: config.display().to_string(),
        env_file: env_file.display().to_string(),
    }
}

pub fn env_get(path: &Path, key: &str) -> ConfigResult<EnvValueRecord> {
    let value = get_env_value(path, key)?;
    Ok(EnvValueRecord {
        key: key.to_string(),
        value,
    })
}

pub fn env_set(path: &Path, key: &str, value: &str) -> ConfigResult<EnvWriteRecord> {
    let EnvWriteResult {
        previous,
        backup_written,
    } = set_env_value(path, key, value)?;
    Ok(EnvWriteRecord {
        path: path.display().to_string(),
        key: key.to_string(),
        previous,
        backup_written,
    })
}

pub fn env_unset(path: &Path, key: &str) -> ConfigResult<EnvUnsetRecord> {
    let previous = unset_env_value(path, key)?;
    Ok(EnvUnsetRecord {
        path: path.display().to_string(),
        key: key.to_string(),
        removed: previous.is_some(),
    })
}

pub fn env_list(path: &Path) -> ConfigResult<EnvListRecord> {
    let keys = list_env_keys(path)?;
    Ok(EnvListRecord {
        path: path.display().to_string(),
        keys,
    })
}

pub fn default_paths() -> AgentPaths {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    AgentPaths::from_home(&home)
}

fn load_or_empty_toml(path: &Path) -> ConfigResult<TomlValue> {
    if path.exists() {
        let raw = fs::read_to_string(path)?;
        Ok(toml::from_str(&raw).map_err(ConfigError::from)?)
    } else {
        Ok(TomlValue::Table(Table::new()))
    }
}

fn validation_error(root: &TomlValue) -> Option<String> {
    value_path::validate_as_agent_config(root)
        .err()
        .map(|err| err.to_string())
}

fn ensure_parent(path: &Path) -> ConfigResult<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn toml_to_json(value: TomlValue) -> JsonValue {
    match value {
        TomlValue::String(s) => JsonValue::String(s),
        TomlValue::Integer(i) => JsonValue::Number(i.into()),
        TomlValue::Float(f) => {
            serde_json::Number::from_f64(f).map_or(JsonValue::Null, JsonValue::Number)
        }
        TomlValue::Boolean(b) => JsonValue::Bool(b),
        TomlValue::Datetime(dt) => JsonValue::String(dt.to_string()),
        TomlValue::Array(items) => JsonValue::Array(items.into_iter().map(toml_to_json).collect()),
        TomlValue::Table(table) => JsonValue::Object(
            table
                .into_iter()
                .map(|(k, v)| (k, toml_to_json(v)))
                .collect(),
        ),
    }
}
