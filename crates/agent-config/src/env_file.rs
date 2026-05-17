use std::{fs, path::Path};

use crate::{ConfigError, ConfigResult};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvWriteResult {
    pub previous: Option<String>,
    pub backup_written: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EnvLine {
    Verbatim(String),
    Entry { key: String, value: String },
}

pub fn set_env_value(path: &Path, key: &str, value: &str) -> ConfigResult<EnvWriteResult> {
    validate_env_key(key)?;
    let (existing, mut lines) = read_lines(path)?;

    let mut previous = None;
    let mut updated = false;
    for line in &mut lines {
        if let EnvLine::Entry { key: k, value: v } = line
            && k == key
        {
            previous = Some(v.clone());
            if v != value {
                *v = value.to_string();
                updated = true;
            }
            break;
        }
    }
    if previous.is_none() {
        lines.push(EnvLine::Entry {
            key: key.to_string(),
            value: value.to_string(),
        });
        updated = true;
    }

    if !updated {
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

    atomic_write(path, &render_lines(&lines))?;
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
    let (existing, mut lines) = read_lines(path)?;
    let mut removed = None;
    lines.retain(|line| match line {
        EnvLine::Entry { key: k, value } if k == key => {
            removed = Some(value.clone());
            false
        }
        _ => true,
    });
    if removed.is_some() {
        fs::write(path.with_extension("env.bak"), &existing)?;
        atomic_write(path, &render_lines(&lines))?;
    }
    Ok(removed)
}

pub fn get_env_value(path: &Path, key: &str) -> ConfigResult<Option<String>> {
    validate_env_key(key)?;
    if !path.exists() {
        return Ok(None);
    }
    let (_, lines) = read_lines(path)?;
    Ok(lines.into_iter().find_map(|line| match line {
        EnvLine::Entry { key: k, value } if k == key => Some(value),
        _ => None,
    }))
}

pub fn list_env_keys(path: &Path) -> ConfigResult<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let (_, lines) = read_lines(path)?;
    Ok(lines
        .into_iter()
        .filter_map(|line| match line {
            EnvLine::Entry { key, .. } => Some(key),
            _ => None,
        })
        .collect())
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

fn read_lines(path: &Path) -> ConfigResult<(String, Vec<EnvLine>)> {
    let raw = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let lines = parse_env_lines(&raw);
    Ok((raw, lines))
}

fn parse_env_lines(raw: &str) -> Vec<EnvLine> {
    raw.lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return EnvLine::Verbatim(line.to_string());
            }
            match trimmed.split_once('=') {
                Some((key, value)) => EnvLine::Entry {
                    key: key.trim().to_string(),
                    value: unquote_env_value(value.trim()),
                },
                None => EnvLine::Verbatim(line.to_string()),
            }
        })
        .collect()
}

fn render_lines(lines: &[EnvLine]) -> String {
    let mut out = String::new();
    for line in lines {
        match line {
            EnvLine::Verbatim(s) => out.push_str(s),
            EnvLine::Entry { key, value } => {
                out.push_str(key);
                out.push('=');
                out.push_str(&quote_env_value(value));
            }
        }
        out.push('\n');
    }
    out
}

fn atomic_write(path: &Path, content: &str) -> ConfigResult<()> {
    let file_name = path.file_name().ok_or_else(|| {
        ConfigError::InvalidConfig(format!(
            "env file path has no file name component: {}",
            path.display()
        ))
    })?;
    let mut tmp_name = file_name.to_os_string();
    tmp_name.push(".tmp");
    let tmp_path = path.with_file_name(tmp_name);

    fs::write(&tmp_path, content)?;
    set_owner_only_permissions(&tmp_path)?;
    if let Err(error) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(ConfigError::Io(error));
    }
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
        return value.to_string();
    }
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            other => quoted.push(other),
        }
    }
    quoted.push('"');
    quoted
}

fn unquote_env_value(value: &str) -> String {
    let Some(inner) = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    else {
        return value.to_string();
    };
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
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
