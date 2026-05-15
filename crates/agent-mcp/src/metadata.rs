use rmcp::model::{
    Prompt, RawResource, RawResourceTemplate, ReadResourceResult, ResourceContents, Tool,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct McpToolMetadata {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub input_schema: Value,
    pub output_schema: Option<Value>,
    pub annotations: Option<Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct McpResourceMetadata {
    pub uri: String,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct McpResourceTemplateMetadata {
    pub uri_template: String,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct McpPromptMetadata {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub arguments: Value,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct McpReadResourceResult {
    pub contents: Vec<McpResourceContent>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpResourceContent {
    Text {
        uri: String,
        mime_type: Option<String>,
        text: String,
    },
    Blob {
        uri: String,
        mime_type: Option<String>,
        blob: String,
    },
}

impl From<Tool> for McpToolMetadata {
    fn from(tool: Tool) -> Self {
        Self {
            name: tool.name.into_owned(),
            title: tool.title,
            description: tool.description.map(|description| description.into_owned()),
            input_schema: Value::Object((*tool.input_schema).clone()),
            output_schema: tool
                .output_schema
                .map(|schema| Value::Object((*schema).clone())),
            annotations: tool
                .annotations
                .and_then(|annotations| serde_json::to_value(annotations).ok()),
        }
    }
}

impl From<rmcp::model::Resource> for McpResourceMetadata {
    fn from(resource: rmcp::model::Resource) -> Self {
        let RawResource {
            uri,
            name,
            title,
            description,
            mime_type,
            size,
            ..
        } = resource.raw;
        Self {
            uri,
            name,
            title,
            description,
            mime_type,
            size,
        }
    }
}

impl From<rmcp::model::ResourceTemplate> for McpResourceTemplateMetadata {
    fn from(template: rmcp::model::ResourceTemplate) -> Self {
        let RawResourceTemplate {
            uri_template,
            name,
            title,
            description,
            mime_type,
            ..
        } = template.raw;
        Self {
            uri_template,
            name,
            title,
            description,
            mime_type,
        }
    }
}

impl From<Prompt> for McpPromptMetadata {
    fn from(prompt: Prompt) -> Self {
        Self {
            name: prompt.name,
            title: prompt.title,
            description: prompt.description,
            arguments: serde_json::to_value(prompt.arguments.unwrap_or_default())
                .unwrap_or(Value::Array(Vec::new())),
        }
    }
}

impl From<ReadResourceResult> for McpReadResourceResult {
    fn from(result: ReadResourceResult) -> Self {
        Self {
            contents: result
                .contents
                .into_iter()
                .map(|content| match content {
                    ResourceContents::TextResourceContents {
                        uri,
                        mime_type,
                        text,
                        ..
                    } => McpResourceContent::Text {
                        uri,
                        mime_type,
                        text,
                    },
                    ResourceContents::BlobResourceContents {
                        uri,
                        mime_type,
                        blob,
                        ..
                    } => McpResourceContent::Blob {
                        uri,
                        mime_type,
                        blob,
                    },
                })
                .collect(),
        }
    }
}
