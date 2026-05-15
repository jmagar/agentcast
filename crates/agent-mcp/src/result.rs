use rmcp::model::{CallToolResult, RawContent};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct McpToolResult {
    pub content: Vec<McpToolContent>,
    pub structured_content: Option<Value>,
    pub is_error: bool,
}

impl McpToolResult {
    pub fn text(&self) -> Option<&str> {
        self.content.iter().find_map(|content| match content {
            McpToolContent::Text { text } => Some(text.as_str()),
            _ => None,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpToolContent {
    Text { text: String },
    Image { mime_type: String, data: String },
    Resource { uri: String, text: Option<String>, blob: Option<String>, mime_type: Option<String> },
    ResourceLink { uri: String, name: String },
    Audio { mime_type: String, data: String },
    Unknown,
}

impl From<CallToolResult> for McpToolResult {
    fn from(result: CallToolResult) -> Self {
        Self {
            content: result
                .content
                .iter()
                .map(|content| match &content.raw {
                    RawContent::Text(text) => McpToolContent::Text {
                        text: text.text.clone(),
                    },
                    RawContent::Image(image) => McpToolContent::Image {
                        mime_type: image.mime_type.clone(),
                        data: image.data.clone(),
                    },
                    RawContent::Audio(audio) => McpToolContent::Audio {
                        mime_type: audio.mime_type.clone(),
                        data: audio.data.clone(),
                    },
                    RawContent::Resource(resource) => match &resource.resource {
                        rmcp::model::ResourceContents::TextResourceContents {
                            uri,
                            mime_type,
                            text,
                            ..
                        } => McpToolContent::Resource {
                            uri: uri.clone(),
                            text: Some(text.clone()),
                            blob: None,
                            mime_type: mime_type.clone(),
                        },
                        rmcp::model::ResourceContents::BlobResourceContents {
                            uri,
                            mime_type,
                            blob,
                            ..
                        } => McpToolContent::Resource {
                            uri: uri.clone(),
                            text: None,
                            blob: Some(blob.clone()),
                            mime_type: mime_type.clone(),
                        },
                    },
                    RawContent::ResourceLink(link) => McpToolContent::ResourceLink {
                        uri: link.uri.clone(),
                        name: link.name.clone(),
                    },
                })
                .collect(),
            structured_content: result.structured_content,
            is_error: result.is_error.unwrap_or(false),
        }
    }
}
