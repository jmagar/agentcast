use super::*;
use rmcp::model::{AnnotateAble, RawResource, Tool};
use serde_json::json;

#[test]
fn converts_tool_metadata_without_sdk_types() {
    let tool: Tool = serde_json::from_value(json!({
        "name": "echo",
        "title": "Echo",
        "description": "Return input",
        "inputSchema": {
            "type": "object"
        }
    }))
    .expect("tool");

    let metadata = McpToolMetadata::from(tool);

    assert_eq!(metadata.name, "echo");
    assert_eq!(metadata.title.as_deref(), Some("Echo"));
    assert_eq!(metadata.input_schema["type"], "object");
}

#[test]
fn converts_resource_metadata() {
    let resource = RawResource::new("file:///tmp/a.txt", "a.txt")
        .with_description("A file")
        .with_mime_type("text/plain")
        .no_annotation();

    let metadata = McpResourceMetadata::from(resource);

    assert_eq!(metadata.uri, "file:///tmp/a.txt");
    assert_eq!(metadata.mime_type.as_deref(), Some("text/plain"));
}
