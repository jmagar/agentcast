use std::{collections::BTreeMap, fs, path::Path};

use crate::{ConfigError, ConfigResult};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvMerge {
    values: BTreeMap<String, String>,
}

impl EnvMerge {
    pub fn new(values: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>) -> Self {
        Self {
            values: values
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvMergeResult {
    pub inserted: Vec<String>,
    pub unchanged: Vec<String>,
    pub backup_written: bool,
}

pub fn merge_env_file(path: &Path, merge: &EnvMerge) -> ConfigResult<EnvMergeResult> {
    for key in merge.values.keys() {
        validate_env_key_for_merge(key)?;
    }

    let existing = fs::read_to_string(path).unwrap_or_default();
    let mut parsed = parse_env_lines(&existing);
    let mut result = EnvMergeResult::default();

    for (key, value) in &merge.values {
        match parsed.get(key) {
            Some(existing) if existing == value => result.unchanged.push(key.clone()),
            Some(_) => return Err(ConfigError::EnvConflict(key.clone())),
            None => {
                parsed.insert(key.clone(), value.clone());
                result.inserted.push(key.clone());
            }
        }
    }

    if !result.inserted.is_empty() {
        if path.exists() {
            fs::write(path.with_extension("env.bak"), existing)?;
            result.backup_written = true;
        }
        let mut rendered = String::new();
        for (key, value) in parsed {
            rendered.push_str(&key);
            rendered.push('=');
            rendered.push_str(&quote_env_value(&value));
            rendered.push('\n');
        }
        fs::write(path, rendered)?;
        set_owner_only_permissions(path)?;
    }

    Ok(result)
}

pub fn validate_env_key_for_merge(key: &str) -> ConfigResult<()> {
    let valid = !key.is_empty()
        && key.starts_with(|ch: char| ch.is_ascii_uppercase())
        && key
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_');
    let sensitive_shape = key.contains("TOKEN")
        || key.contains("SECRET")
        || key.contains("KEY")
        || key.contains("URL")
        || key.ends_with("_ENV")
        || key.starts_with("MCP_");
    if valid && sensitive_shape {
        Ok(())
    } else {
        Err(ConfigError::InvalidConfig(format!(
            "env merge key `{key}` must be an uppercase token/secret/key/url/runtime env reference"
        )))
    }
}

fn parse_env_lines(raw: &str) -> BTreeMap<String, String> {
    raw.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            Some((key.trim().to_string(), unquote_env_value(value.trim())))
        })
        .collect()
}

fn quote_env_value(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | '/' | ':'))
    {
        value.to_string()
    } else {
        format!("{value:?}")
    }
}

fn unquote_env_value(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
        .to_string()
}

#[cfg(unix)]
fn set_owner_only_permissions(path: &Path) -> ConfigResult<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_owner_only_permissions(_path: &Path) -> ConfigResult<()> {
    Ok(())
}
