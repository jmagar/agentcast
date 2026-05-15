use serde_json::{Map, Value};

pub fn expect_json_object(value: Value) -> Result<Map<String, Value>, Error> {
    match value {
        Value::Object(object) => Ok(object),
        _ => Err(Error::ExpectedObject),
    }
}

pub fn optional_json_string<'a>(
    object: &'a Map<String, Value>,
    key: &str,
) -> Result<Option<&'a str>, Error> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.as_str())),
        Some(_) => Err(Error::ExpectedString),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("expected JSON object")]
    ExpectedObject,
    #[error("expected JSON string")]
    ExpectedString,
}
