use std::{collections::BTreeMap, fs, path::Path};

use crate::{ConfigError, ConfigResult};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvWriteResult {
    pub previous: Option<String>,
    pub backup_written: bool,
}

pub fn set_env_value(path: &Path, key: &str, value: &str) -> ConfigResult<EnvWriteResult> {
    validate_env_key(key)?;
    let existing = fs::read_to_string(path).unwrap_or_default();
    let mut parsed = parse_env_lines(&existing);
    let previous = parsed.insert(key.to_string(), value.to_string());

    if previous.as_deref() == Some(value) {
        return Ok(EnvWriteResult {
            previous,
            backup_written: false,
        });
    }

    let backup_written = if path.exists() {
        fs::write(path.with_extension("env.bak"), &existing)?;
        true
    } else {
        ensure_parent(path)?;
        false
    };

    write_env(path, &parsed)?;
    Ok(EnvWriteResult {
        previous,
        backup_written,
    })
}

pub fn unset_env_value(path: &Path, key: &str) -> ConfigResult<Option<String>> {
    validate_env_key(key)?;
    if !path.exists() {
        return Ok(None);
    }
    let existing = fs::read_to_string(path)?;
    let mut parsed = parse_env_lines(&existing);
    let removed = parsed.remove(key);
    if removed.is_some() {
        fs::write(path.with_extension("env.bak"), &existing)?;
        write_env(path, &parsed)?;
    }
    Ok(removed)
}

pub fn get_env_value(path: &Path, key: &str) -> ConfigResult<Option<String>> {
    validate_env_key(key)?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)?;
    Ok(parse_env_lines(&raw).remove(key))
}

pub fn list_env_keys(path: &Path) -> ConfigResult<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path)?;
    Ok(parse_env_lines(&raw).into_keys().collect())
}

fn validate_env_key(key: &str) -> ConfigResult<()> {
    let valid = !key.is_empty()
        && key.starts_with(|ch: char| ch.is_ascii_uppercase() || ch == '_')
        && key
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_');
    if valid {
        Ok(())
    } else {
        Err(ConfigError::InvalidConfig(format!(
            "env key `{key}` must match ^[A-Z_][A-Z0-9_]*$"
        )))
    }
}

fn parse_env_lines(raw: &str) -> BTreeMap<String, String> {
    raw.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return None;
            }
            let (key, value) = trimmed.split_once('=')?;
            Some((key.trim().to_string(), unquote_env_value(value.trim())))
        })
        .collect()
}

fn write_env(path: &Path, entries: &BTreeMap<String, String>) -> ConfigResult<()> {
    let mut rendered = String::new();
    for (key, value) in entries {
        rendered.push_str(key);
        rendered.push('=');
        rendered.push_str(&quote_env_value(value));
        rendered.push('\n');
    }
    fs::write(path, rendered)?;
    set_owner_only_permissions(path)?;
    Ok(())
}

fn ensure_parent(path: &Path) -> ConfigResult<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    Ok(())
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
