use std::collections::BTreeSet;

use serde_json::Value;

use crate::{NormalizedField, NormalizedSchema, SchemaError, SchemaKind, SchemaResult};

#[cfg(test)]
mod tests;

pub fn normalize_schema(schema: &Value) -> SchemaResult<NormalizedSchema> {
    let object = schema
        .as_object()
        .ok_or_else(|| SchemaError::InvalidSchema("root schema must be an object".into()))?;
    let required = object
        .get("required")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let fields = object
        .get("properties")
        .and_then(Value::as_object)
        .map(|properties| {
            properties
                .iter()
                .map(|(name, schema)| normalize_field(name, schema, required.contains(name)))
                .collect::<SchemaResult<Vec<_>>>()
        })
        .transpose()?
        .unwrap_or_default();
    Ok(NormalizedSchema {
        title: object
            .get("title")
            .and_then(Value::as_str)
            .map(str::to_string),
        fields,
    })
}

fn normalize_field(name: &str, schema: &Value, required: bool) -> SchemaResult<NormalizedField> {
    let object = schema
        .as_object()
        .ok_or_else(|| SchemaError::InvalidSchema(format!("field `{name}` must be an object")))?;
    let kind = match object.get("type").and_then(Value::as_str) {
        Some("object") => SchemaKind::Object,
        Some("string") => SchemaKind::String,
        Some("number") => SchemaKind::Number,
        Some("integer") => SchemaKind::Integer,
        Some("boolean") => SchemaKind::Boolean,
        Some("array") => SchemaKind::Array,
        _ => SchemaKind::Unsupported,
    };
    let nested_required = object
        .get("required")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let fields = object
        .get("properties")
        .and_then(Value::as_object)
        .map(|properties| {
            properties
                .iter()
                .map(|(name, schema)| normalize_field(name, schema, nested_required.contains(name)))
                .collect::<SchemaResult<Vec<_>>>()
        })
        .transpose()?
        .unwrap_or_default();
    Ok(NormalizedField {
        name: name.to_string(),
        kind,
        required,
        title: object
            .get("title")
            .and_then(Value::as_str)
            .map(str::to_string),
        description: object
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_string),
        default: object.get("default").cloned(),
        enum_values: object
            .get("enum")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        fields,
    })
}
