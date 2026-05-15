use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct NormalizedSchema {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub fields: Vec<NormalizedField>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct NormalizedField {
    pub name: String,
    pub kind: SchemaKind,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub default: Option<Value>,
    #[serde(default)]
    pub enum_values: Vec<String>,
    #[serde(default)]
    pub fields: Vec<NormalizedField>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaKind {
    Object,
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Unsupported,
}
