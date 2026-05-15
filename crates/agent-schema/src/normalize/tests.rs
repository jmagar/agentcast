use super::*;
use serde_json::json;

#[test]
fn normalizes_required_default_enum_and_nested_fields() {
    let schema = json!({
        "title": "Input",
        "type": "object",
        "required": ["path"],
        "properties": {
            "path": {"type": "string", "description": "File path"},
            "mode": {"type": "string", "enum": ["read", "write"], "default": "read"},
            "options": {
                "type": "object",
                "required": ["recursive"],
                "properties": {"recursive": {"type": "boolean"}}
            }
        }
    });

    let normalized = normalize_schema(&schema).unwrap();
    assert_eq!(normalized.title.as_deref(), Some("Input"));
    assert!(
        normalized
            .fields
            .iter()
            .find(|field| field.name == "path")
            .unwrap()
            .required
    );
    assert_eq!(
        normalized
            .fields
            .iter()
            .find(|field| field.name == "mode")
            .unwrap()
            .enum_values,
        ["read", "write"]
    );
    assert_eq!(
        normalized
            .fields
            .iter()
            .find(|field| field.name == "options")
            .unwrap()
            .fields[0]
            .kind,
        SchemaKind::Boolean
    );
}
