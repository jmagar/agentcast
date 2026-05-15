use serde_json::Value;

use crate::{NormalizedField, NormalizedSchema, SchemaError, SchemaKind, SchemaResult};

#[cfg(test)]
mod tests;

pub fn validate_payload(schema: &NormalizedSchema, payload: &Value) -> SchemaResult<()> {
    let object = payload
        .as_object()
        .ok_or_else(|| SchemaError::InvalidPayload("payload must be an object".into()))?;
    for field in &schema.fields {
        match object.get(&field.name) {
            Some(value) => validate_field(field, value)?,
            None if field.required => {
                return Err(SchemaError::InvalidPayload(format!(
                    "missing required field `{}`",
                    field.name
                )));
            }
            None => {}
        }
    }
    Ok(())
}

fn validate_field(field: &NormalizedField, value: &Value) -> SchemaResult<()> {
    let valid = match field.kind {
        SchemaKind::Object => value.is_object(),
        SchemaKind::String => value.is_string(),
        SchemaKind::Number => value.is_number(),
        SchemaKind::Integer => value.as_i64().is_some() || value.as_u64().is_some(),
        SchemaKind::Boolean => value.is_boolean(),
        SchemaKind::Array => value.is_array(),
        SchemaKind::Unsupported => true,
    };
    if !valid {
        return Err(SchemaError::InvalidPayload(format!(
            "field `{}` has wrong type",
            field.name
        )));
    }
    if !field.enum_values.is_empty()
        && value
            .as_str()
            .is_some_and(|value| !field.enum_values.iter().any(|allowed| allowed == value))
    {
        return Err(SchemaError::InvalidPayload(format!(
            "field `{}` is not an allowed enum value",
            field.name
        )));
    }
    Ok(())
}
