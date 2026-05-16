use serde_json::Value as JsonValue;
use toml::{Table, Value as TomlValue};

use crate::{AgentConfig, ConfigError, ConfigResult};

#[cfg(test)]
mod tests;

pub fn get_value(root: &TomlValue, path: &str) -> ConfigResult<Option<TomlValue>> {
    let segments = parse_path(path)?;
    Ok(navigate(root, &segments).cloned())
}

pub fn set_value(root: &mut TomlValue, path: &str, raw: &str) -> ConfigResult<()> {
    let segments = parse_path(path)?;
    let value = parse_scalar_value(raw)?;
    insert_at_path(root, &segments, value)
}

pub fn unset_value(root: &mut TomlValue, path: &str) -> ConfigResult<bool> {
    let segments = parse_path(path)?;
    remove_at_path(root, &segments)
}

pub fn validate_as_agent_config(root: &TomlValue) -> ConfigResult<AgentConfig> {
    let mut config: AgentConfig = root.clone().try_into()?;
    config.validate()?;
    Ok(config)
}

fn parse_path(path: &str) -> ConfigResult<Vec<String>> {
    if path.trim().is_empty() {
        return Err(ConfigError::InvalidConfig(
            "config path cannot be empty".into(),
        ));
    }
    let segments: Vec<String> = path.split('.').map(str::to_string).collect();
    if segments.iter().any(String::is_empty) {
        return Err(ConfigError::InvalidConfig(format!(
            "config path `{path}` has an empty segment"
        )));
    }
    Ok(segments)
}

fn parse_scalar_value(raw: &str) -> ConfigResult<TomlValue> {
    match serde_json::from_str::<JsonValue>(raw) {
        Ok(JsonValue::Null) => Err(ConfigError::InvalidConfig(
            "cannot set value to null; use `config unset` to remove a key".into(),
        )),
        Ok(value) => json_to_toml(value),
        Err(_) => Ok(TomlValue::String(raw.to_string())),
    }
}

fn json_to_toml(value: JsonValue) -> ConfigResult<TomlValue> {
    Ok(match value {
        JsonValue::Null => {
            return Err(ConfigError::InvalidConfig(
                "null is not representable in config TOML".into(),
            ));
        }
        JsonValue::Bool(b) => TomlValue::Boolean(b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                TomlValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                TomlValue::Float(f)
            } else {
                return Err(ConfigError::InvalidConfig(format!(
                    "numeric value `{n}` is not representable in TOML"
                )));
            }
        }
        JsonValue::String(s) => TomlValue::String(s),
        JsonValue::Array(items) => {
            let mut converted = Vec::with_capacity(items.len());
            for item in items {
                converted.push(json_to_toml(item)?);
            }
            TomlValue::Array(converted)
        }
        JsonValue::Object(map) => {
            let mut table = Table::new();
            for (key, value) in map {
                table.insert(key, json_to_toml(value)?);
            }
            TomlValue::Table(table)
        }
    })
}

fn navigate<'a>(root: &'a TomlValue, segments: &[String]) -> Option<&'a TomlValue> {
    let mut current = root;
    for segment in segments {
        match current {
            TomlValue::Table(table) => current = table.get(segment)?,
            _ => return None,
        }
    }
    Some(current)
}

fn insert_at_path(root: &mut TomlValue, segments: &[String], value: TomlValue) -> ConfigResult<()> {
    let mut current = root;
    for (index, segment) in segments.iter().enumerate() {
        let is_last = index == segments.len() - 1;
        let table = match current {
            TomlValue::Table(table) => table,
            _ => {
                return Err(ConfigError::InvalidConfig(format!(
                    "cannot descend into non-table value at segment `{segment}`"
                )));
            }
        };
        if is_last {
            table.insert(segment.clone(), value);
            return Ok(());
        }
        let entry = table
            .entry(segment.clone())
            .or_insert_with(|| TomlValue::Table(Table::new()));
        if !entry.is_table() {
            return Err(ConfigError::InvalidConfig(format!(
                "cannot descend into non-table value at segment `{segment}`"
            )));
        }
        current = entry;
    }
    Ok(())
}

fn remove_at_path(root: &mut TomlValue, segments: &[String]) -> ConfigResult<bool> {
    let (last, parents) = segments.split_last().expect("path validated as non-empty");
    let mut current = root;
    for segment in parents {
        let table = match current {
            TomlValue::Table(table) => table,
            _ => return Ok(false),
        };
        current = match table.get_mut(segment) {
            Some(value) => value,
            None => return Ok(false),
        };
    }
    match current {
        TomlValue::Table(table) => Ok(table.remove(last).is_some()),
        _ => Ok(false),
    }
}
