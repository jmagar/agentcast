use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use url::Url;

use crate::{MarketplaceError, MarketplaceResult};

#[cfg(test)]
mod tests;

const ALLOWED_RUNTIMES: &[&str] = &[
    "npx", "uvx", "docker", "dnx", "pipx", "node", "python", "python3", "deno",
];

const PROTECTED_ENV_NAMES: &[&str] = &[
    "PATH",
    "LD_PRELOAD",
    "LD_LIBRARY_PATH",
    "HOME",
    "SHELL",
    "IFS",
    "USER",
    "PWD",
];

const DANGEROUS_DOCKER_FLAGS: &[&str] = &[
    "--privileged",
    "--cap-add",
    "--volume",
    "-v",
    "--device",
    "--network",
    "--pid",
    "--ipc",
];

const DANGEROUS_NODE_FLAGS: &[&str] =
    &["--inspect", "--require", "-r", "--experimental", "--allow"];

pub fn validate_runtime_hint(value: &str) -> MarketplaceResult<()> {
    if ALLOWED_RUNTIMES.contains(&value) {
        Ok(())
    } else {
        Err(MarketplaceError::UnsafeParameter(format!(
            "runtime hint `{value}` is not allowlisted"
        )))
    }
}

pub fn validate_stdio_argv(runtime_hint: &str, args: &[String]) -> MarketplaceResult<()> {
    for arg in args {
        if arg.contains('\n') || arg.contains('\r') || arg.contains('\0') {
            return Err(MarketplaceError::UnsafeParameter(
                "argv values must not contain newline, carriage return, or null bytes".into(),
            ));
        }
        validate_runtime_argv_flag(runtime_hint, arg)?;
    }
    Ok(())
}

fn validate_runtime_argv_flag(runtime_hint: &str, arg: &str) -> MarketplaceResult<()> {
    let flag = arg.split('=').next().unwrap_or(arg);
    let denied = match runtime_hint {
        "docker" => {
            DANGEROUS_DOCKER_FLAGS.contains(&flag)
                || matches!(arg, "--network=host" | "--pid=host" | "--ipc=host")
        }
        "node" | "npx" => DANGEROUS_NODE_FLAGS
            .iter()
            .any(|prefix| flag == *prefix || flag.starts_with(*prefix)),
        _ => false,
    };

    if denied {
        Err(MarketplaceError::UnsafeParameter(format!(
            "argv flag `{arg}` is not allowed for runtime hint `{runtime_hint}`"
        )))
    } else {
        Ok(())
    }
}

pub fn validate_env_name(value: &str) -> MarketplaceResult<()> {
    let valid_name = !value.is_empty()
        && value.starts_with(|ch: char| ch.is_ascii_uppercase())
        && value
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_');
    let protected = PROTECTED_ENV_NAMES.contains(&value) || value.starts_with("AGENTCAST_");

    if valid_name && !protected {
        Ok(())
    } else {
        Err(MarketplaceError::UnsafeParameter(format!(
            "env var `{value}` is not allowed"
        )))
    }
}

pub fn validate_env_value(value: &str) -> MarketplaceResult<()> {
    if value.contains('\n') || value.contains('\r') || value.contains('\0') {
        return Err(MarketplaceError::UnsafeParameter(
            "env value cannot contain newline, carriage return, or null bytes".into(),
        ));
    }
    Ok(())
}

pub fn validate_registry_url(value: &str) -> MarketplaceResult<Url> {
    let url = Url::parse(value)
        .map_err(|error| MarketplaceError::UnsafeParameter(format!("invalid URL: {error}")))?;
    if !matches!(url.scheme(), "https" | "http") {
        return Err(MarketplaceError::UnsafeParameter(
            "registry URL must use http or https".into(),
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(MarketplaceError::UnsafeParameter(
            "registry URL must not include credentials".into(),
        ));
    }
    if url.scheme() != "https" && !is_loopback_host(&url) {
        return Err(MarketplaceError::UnsafeParameter(
            "registry URL must use https unless it targets loopback".into(),
        ));
    }
    if let Some(host) = url.host_str()
        && let Ok(ip) = host.parse::<IpAddr>()
    {
        reject_private_ip(ip)?;
    }
    Ok(url)
}

fn is_loopback_host(url: &Url) -> bool {
    matches!(url.host_str(), Some("localhost"))
        || url
            .host_str()
            .and_then(|host| host.parse::<IpAddr>().ok())
            .is_some_and(|ip| ip.is_loopback())
}

fn reject_private_ip(ip: IpAddr) -> MarketplaceResult<()> {
    let unsafe_ip = match ip {
        IpAddr::V4(ip) => is_private_v4(ip),
        IpAddr::V6(ip) => is_private_v6(ip),
    };
    if unsafe_ip {
        Err(MarketplaceError::UnsafeParameter(format!(
            "registry URL host `{ip}` is not public"
        )))
    } else {
        Ok(())
    }
}

fn is_private_v4(ip: Ipv4Addr) -> bool {
    ip.is_private()
        || ip.is_loopback()
        || ip.is_link_local()
        || ip.is_broadcast()
        || ip.is_documentation()
        || ip.octets()[0] == 0
}

fn is_private_v6(ip: Ipv6Addr) -> bool {
    ip.is_loopback() || ip.is_unique_local() || ip.is_unicast_link_local() || ip.is_unspecified()
}
