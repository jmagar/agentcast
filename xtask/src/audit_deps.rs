#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use crate::{Error, Result};

const FORBIDDEN_DEPENDENCIES: &[ForbiddenDependency] = &[
    ForbiddenDependency::new(
        "agent-protocol",
        "agent-mcp",
        "protocol/model crates must stay adapter-neutral",
    ),
    ForbiddenDependency::new(
        "agent-protocol",
        "agent-acp",
        "protocol/model crates must stay adapter-neutral",
    ),
];

const SURFACE_CRATES: &[&str] = &[
    "agent-api",
    "agent-cli",
    "agent-server",
    "agent-ui-contracts",
];
const PROTOCOL_ADAPTERS: &[&str] = &["agent-mcp", "agent-acp"];
const SURFACE_ADAPTER_EXCEPTIONS: &[(&str, &str, &str)] = &[(
    "agent-server",
    "agent-mcp",
    "server binary owns the temporary v0 stdio gateway composition until that adapter boundary is split",
)];

const LOW_LEVEL_FORBIDDEN: &[(&str, &[&str], &str)] = &[
    (
        "agent-core",
        &[
            "tokio",
            "axum",
            "clap",
            "rmcp",
            "agent-client-protocol",
            "reqwest",
            "rusqlite",
        ],
        "agent-core must stay dependency-light",
    ),
    (
        "agent-protocol",
        &["rmcp", "agent-client-protocol", "axum", "clap"],
        "agent-protocol must stay protocol-SDK and surface neutral",
    ),
];

const EXCLUSIVE_OWNERS: &[(&str, &str, &str)] = &[
    ("rmcp", "agent-mcp", "rmcp belongs in the MCP adapter"),
    (
        "agent-client-protocol",
        "agent-acp",
        "agent-client-protocol belongs in the ACP adapter",
    ),
];

#[derive(Clone, Copy)]
struct ForbiddenDependency {
    crate_name: &'static str,
    dependency: &'static str,
    rule: &'static str,
}

impl ForbiddenDependency {
    const fn new(crate_name: &'static str, dependency: &'static str, rule: &'static str) -> Self {
        Self {
            crate_name,
            dependency,
            rule,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ManifestDeps {
    package: String,
    dependencies: Vec<String>,
}

pub fn run() -> Result<()> {
    let root = std::env::current_dir().map_err(Error::Io)?;
    let manifests = manifest_paths(&root)?;
    let mut findings = Vec::new();

    for manifest_path in &manifests {
        let content = fs::read_to_string(manifest_path).map_err(Error::Io)?;
        let manifest = parse_manifest_dependencies(&content).map_err(|message| {
            Error::DependencyAudit(vec![format!("{}: {message}", manifest_path.display())])
        })?;
        if let Some(manifest) = manifest {
            check_manifest(&manifest, &mut findings);
        }
    }

    println!("audit-deps: checked {} manifests", manifests.len());

    if findings.is_empty() {
        Ok(())
    } else {
        Err(Error::DependencyAudit(findings))
    }
}

fn manifest_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let mut manifests = vec![root.join("Cargo.toml"), root.join("xtask/Cargo.toml")];
    let crates_dir = root.join("crates");
    if crates_dir.is_dir() {
        for entry in fs::read_dir(crates_dir).map_err(Error::Io)? {
            let entry = entry.map_err(Error::Io)?;
            let manifest = entry.path().join("Cargo.toml");
            if manifest.is_file() {
                manifests.push(manifest);
            }
        }
    }
    manifests.sort();
    Ok(manifests)
}

fn check_manifest(manifest: &ManifestDeps, findings: &mut Vec<String>) {
    for forbidden in FORBIDDEN_DEPENDENCIES {
        if manifest.package == forbidden.crate_name && manifest.has_dependency(forbidden.dependency)
        {
            findings.push(format!(
                "{}: direct dependency on {} violates {}",
                manifest.package, forbidden.dependency, forbidden.rule
            ));
        }
    }

    if SURFACE_CRATES.contains(&manifest.package.as_str()) {
        for dependency in PROTOCOL_ADAPTERS {
            if manifest.has_dependency(dependency)
                && surface_adapter_exception(&manifest.package, dependency).is_none()
            {
                findings.push(format!(
                    "{}: direct dependency on {} violates surface crates must not invoke protocol SDKs directly",
                    manifest.package, dependency
                ));
            }
        }
    }

    for (crate_name, dependencies, rule) in LOW_LEVEL_FORBIDDEN {
        if manifest.package == *crate_name {
            for dependency in *dependencies {
                if manifest.has_dependency(dependency) {
                    findings.push(format!(
                        "{}: direct dependency on {} violates {}",
                        manifest.package, dependency, rule
                    ));
                }
            }
        }
    }

    for (dependency, owner, rule) in EXCLUSIVE_OWNERS {
        if manifest.package != *owner && manifest.has_dependency(dependency) {
            findings.push(format!(
                "{}: direct dependency on {} violates {}",
                manifest.package, dependency, rule
            ));
        }
    }
}

fn surface_adapter_exception(crate_name: &str, dependency: &str) -> Option<&'static str> {
    SURFACE_ADAPTER_EXCEPTIONS
        .iter()
        .find(|(allowed_crate, allowed_dependency, _)| {
            *allowed_crate == crate_name && *allowed_dependency == dependency
        })
        .map(|(_, _, reason)| *reason)
}

fn parse_manifest_dependencies(content: &str) -> std::result::Result<Option<ManifestDeps>, String> {
    let mut package = None;
    let mut dependencies = Vec::new();
    let mut in_package = false;
    let mut in_dependency_section = false;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            let section = line.trim_matches(&['[', ']'][..]);
            in_package = section == "package";
            in_dependency_section = section == "dependencies"
                || section == "dev-dependencies"
                || section == "build-dependencies"
                || section.ends_with(".dependencies")
                || section.ends_with(".dev-dependencies")
                || section.ends_with(".build-dependencies");
            continue;
        }

        if in_package
            && package.is_none()
            && let Some(value) = assignment_value(line, "name")
        {
            package = Some(value.to_owned());
        }

        if in_dependency_section && let Some((name, _)) = line.split_once('=') {
            dependencies.push(name.trim().trim_matches('"').to_owned());
        }
    }

    let Some(package) = package else {
        return Ok(None);
    };

    dependencies.sort();
    dependencies.dedup();
    Ok(Some(ManifestDeps {
        package,
        dependencies,
    }))
}

fn assignment_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let (name, value) = line.split_once('=')?;
    if name.trim() != key {
        return None;
    }
    Some(value.trim().trim_matches('"'))
}

impl ManifestDeps {
    fn has_dependency(&self, dependency: &str) -> bool {
        self.dependencies.iter().any(|name| name == dependency)
    }
}
