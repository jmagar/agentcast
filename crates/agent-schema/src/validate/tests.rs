use super::*;
use crate::normalize_schema;
use serde_json::json;

#[test]
fn validates_required_fields_and_enum_values() {
    let normalized = normalize_schema(&json!({
        "type": "object",
        "required": ["mode"],
        "properties": {"mode": {"type": "string", "enum": ["read"]}}
    }))
    .unwrap();

    assert!(validate_payload(&normalized, &json!({"mode": "read"})).is_ok());
    assert_eq!(
        validate_payload(&normalized, &json!({}))
            .unwrap_err()
            .kind(),
        "invalid_payload"
    );
    assert!(validate_payload(&normalized, &json!({"mode": "write"})).is_err());
}
