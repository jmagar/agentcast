use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::{RegistryError, RegistryResult};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpRegistryResponse {
    #[serde(default)]
    pub servers: Vec<McpRegistryServerResponse>,
    #[serde(default)]
    pub metadata: RegistryPagination,
}

impl McpRegistryResponse {
    pub fn normalize(self) -> RegistryResult<Vec<NormalizedMcpServer>> {
        self.servers
            .into_iter()
            .map(McpRegistryServerResponse::normalize)
            .collect()
    }

    pub fn next_cursor(&self) -> Option<&str> {
        self.metadata.next_cursor.as_deref()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RegistryPagination {
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum McpRegistryServerResponse {
    Official {
        server: McpRegistryServer,
        #[serde(default)]
        meta: Option<RegistryResponseMeta>,
    },
    Flat(McpRegistryServer),
}

impl McpRegistryServerResponse {
    pub fn normalize(self) -> RegistryResult<NormalizedMcpServer> {
        match self {
            Self::Official { server, meta } => server.normalize(meta),
            Self::Flat(server) => server.normalize(None),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpRegistryServer {
    pub name: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub packages: Vec<McpRegistryPackage>,
    #[serde(default)]
    pub remotes: Vec<McpRegistryRemote>,
    #[serde(default)]
    pub repository: Option<McpRegistryRepository>,
    #[serde(default)]
    pub icons: Vec<McpRegistryIcon>,
    #[serde(alias = "websiteUrl", default)]
    pub website_url: Option<String>,
}

impl McpRegistryServer {
    fn normalize(self, meta: Option<RegistryResponseMeta>) -> RegistryResult<NormalizedMcpServer> {
        let name = trim_required("server name", self.name)?;
        Ok(NormalizedMcpServer {
            name,
            title: self.title.and_then(trim_optional),
            description: self.description.and_then(trim_optional),
            latest_version: self.version.and_then(trim_optional),
            packages: self
                .packages
                .into_iter()
                .map(McpRegistryPackage::normalize)
                .collect::<RegistryResult<_>>()?,
            remotes: self
                .remotes
                .into_iter()
                .map(McpRegistryRemote::normalize)
                .collect::<RegistryResult<_>>()?,
            repository_url: self
                .repository
                .and_then(|repository| trim_optional(repository.url)),
            website_url: self.website_url.and_then(trim_optional),
            provenance: RegistryProvenance::official_mcp(),
            registry_metadata: meta
                .and_then(|meta| meta.official)
                .map(NormalizedRegistryMetadata::from)
                .unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpRegistryPackage {
    #[serde(alias = "registryType")]
    pub registry_type: String,
    pub identifier: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub transport: Option<McpRegistryTransport>,
    #[serde(alias = "runtimeHint", default)]
    pub runtime_hint: Option<String>,
    #[serde(alias = "runtimeArguments", default)]
    pub runtime_arguments: Vec<Value>,
    #[serde(alias = "packageArguments", default)]
    pub package_arguments: Vec<Value>,
    #[serde(alias = "environmentVariables", default)]
    pub environment_variables: Vec<McpEnvironmentVariable>,
    #[serde(alias = "fileSha256", default)]
    pub file_sha256: Option<String>,
    #[serde(alias = "registryBaseUrl", default)]
    pub registry_base_url: Option<String>,
}

impl McpRegistryPackage {
    fn normalize(self) -> RegistryResult<NormalizedMcpPackage> {
        Ok(NormalizedMcpPackage {
            registry_type: trim_required("package registry type", self.registry_type)?,
            identifier: trim_required("package identifier", self.identifier)?,
            version: self.version.and_then(trim_optional),
            runtime_hint: self.runtime_hint.and_then(trim_optional),
            transport: self
                .transport
                .and_then(McpRegistryTransport::transport_type),
            runtime_arguments: self.runtime_arguments,
            package_arguments: self.package_arguments,
            environment_variables: self
                .environment_variables
                .into_iter()
                .map(McpEnvironmentVariable::normalize)
                .collect::<RegistryResult<_>>()?,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum McpRegistryTransport {
    Object {
        #[serde(alias = "type")]
        transport_type: String,
        #[serde(default)]
        url: Option<String>,
        #[serde(default)]
        headers: Vec<McpRegistryHeader>,
        #[serde(default)]
        variables: Option<Value>,
    },
    String(String),
}

impl McpRegistryTransport {
    pub fn transport_type(self) -> Option<String> {
        match self {
            Self::Object { transport_type, .. } | Self::String(transport_type) => {
                trim_optional(transport_type)
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpRegistryRemote {
    #[serde(alias = "type")]
    pub transport_type: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub headers: Vec<McpRegistryHeader>,
}

impl McpRegistryRemote {
    fn normalize(self) -> RegistryResult<NormalizedMcpRemote> {
        Ok(NormalizedMcpRemote {
            transport_type: trim_required("remote transport type", self.transport_type)?,
            url: self.url.and_then(trim_optional),
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpRegistryHeader {
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(alias = "isRequired", default)]
    pub is_required: Option<bool>,
    #[serde(alias = "isSecret", default)]
    pub is_secret: Option<bool>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub choices: Vec<String>,
    #[serde(default)]
    pub variables: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpEnvironmentVariable {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(alias = "isRequired", default)]
    pub is_required: bool,
    #[serde(alias = "isSecret", default)]
    pub is_secret: bool,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub choices: Vec<String>,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
}

impl McpEnvironmentVariable {
    fn normalize(self) -> RegistryResult<NormalizedMcpEnvVar> {
        Ok(NormalizedMcpEnvVar {
            name: trim_required("environment variable name", self.name)?,
            description: self.description.and_then(trim_optional),
            is_required: self.is_required,
            is_secret: self.is_secret,
            default: self.default.and_then(trim_optional),
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpRegistryRepository {
    pub url: String,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpRegistryIcon {
    #[serde(alias = "mimeType", default)]
    pub mime_type: Option<String>,
    #[serde(alias = "src")]
    pub url: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RegistryResponseMeta {
    #[serde(default)]
    pub official: Option<RegistryOfficialMeta>,
    #[serde(flatten)]
    pub extensions: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RegistryOfficialMeta {
    #[serde(alias = "isLatest", default)]
    pub is_latest: Option<bool>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(alias = "publishedAt", default)]
    pub published_at: Option<String>,
    #[serde(alias = "updatedAt", default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct NormalizedMcpServer {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub latest_version: Option<String>,
    pub packages: Vec<NormalizedMcpPackage>,
    pub remotes: Vec<NormalizedMcpRemote>,
    pub repository_url: Option<String>,
    pub website_url: Option<String>,
    pub provenance: RegistryProvenance,
    pub registry_metadata: NormalizedRegistryMetadata,
}

impl NormalizedMcpServer {
    pub fn cache_key(&self) -> String {
        format!("mcp:{}", self.name)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RegistryProvenance {
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
}

impl RegistryProvenance {
    #[must_use]
    pub fn official_mcp() -> Self {
        Self {
            source: "mcp_registry".to_string(),
            source_url: Some("https://registry.modelcontextprotocol.io".to_string()),
        }
    }
}

impl Default for RegistryProvenance {
    fn default() -> Self {
        Self::official_mcp()
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct NormalizedRegistryMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_latest: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

impl From<RegistryOfficialMeta> for NormalizedRegistryMetadata {
    fn from(value: RegistryOfficialMeta) -> Self {
        Self {
            is_latest: value.is_latest,
            status: value.status.and_then(trim_optional),
            published_at: value.published_at.and_then(trim_optional),
            updated_at: value.updated_at.and_then(trim_optional),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct NormalizedMcpPackage {
    pub registry_type: String,
    pub identifier: String,
    pub version: Option<String>,
    pub runtime_hint: Option<String>,
    pub transport: Option<String>,
    pub runtime_arguments: Vec<Value>,
    pub package_arguments: Vec<Value>,
    pub environment_variables: Vec<NormalizedMcpEnvVar>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct NormalizedMcpEnvVar {
    pub name: String,
    pub description: Option<String>,
    pub is_required: bool,
    pub is_secret: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct NormalizedMcpRemote {
    pub transport_type: String,
    pub url: Option<String>,
}

fn trim_required(field: &str, value: String) -> RegistryResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(RegistryError::InvalidData(format!(
            "{field} cannot be blank"
        )));
    }
    Ok(value.to_string())
}

fn trim_optional(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}
