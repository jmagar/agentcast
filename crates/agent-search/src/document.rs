use agent_protocol::LauncherActionId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const MAX_SEARCH_FIELD_CHARS: usize = 256;
const REDACTED: &str = "[REDACTED]";

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct SearchDocument {
    pub action_id: LauncherActionId,
    pub name: String,
    pub description: Option<String>,
    pub metadata: Vec<String>,
    #[serde(default)]
    pub schema_summary: Option<String>,
    pub catalog_hash: String,
    pub truncated: bool,
}

impl SearchDocument {
    pub fn sanitized(mut self) -> Self {
        let (name, name_truncated) = sanitize_field(self.name);
        self.name = name;
        self.truncated |= name_truncated;
        self.description = self.description.map(|description| {
            let (description, truncated) = sanitize_field(description);
            self.truncated |= truncated;
            description
        });
        self.metadata = self
            .metadata
            .into_iter()
            .map(|metadata| {
                let (metadata, truncated) = sanitize_field(metadata);
                self.truncated |= truncated;
                metadata
            })
            .collect();
        self.schema_summary = self.schema_summary.map(|summary| {
            let (summary, truncated) = sanitize_field(summary);
            self.truncated |= truncated;
            summary
        });
        self
    }

    pub fn searchable_text(&self) -> String {
        let mut parts = vec![self.name.clone()];
        if let Some(description) = &self.description {
            parts.push(description.clone());
        }
        parts.extend(self.metadata.iter().cloned());
        if let Some(summary) = &self.schema_summary {
            parts.push(summary.clone());
        }
        parts.join(" ")
    }
}

fn sanitize_field(raw: String) -> (String, bool) {
    let (mut value, redacted) = if is_secret_like(&raw) {
        (REDACTED.to_string(), true)
    } else {
        (raw, false)
    };

    if value.chars().count() > MAX_SEARCH_FIELD_CHARS {
        value = value.chars().take(MAX_SEARCH_FIELD_CHARS).collect();
        value.push_str("...");
        return (value, true);
    }

    (value, redacted)
}

fn is_secret_like(raw: &str) -> bool {
    let lower = raw.to_ascii_lowercase();
    let secret_key = [
        "authorization",
        "bearer ",
        "api_key",
        "apikey",
        "access_token",
        "refresh_token",
        "password",
        "secret",
        "token",
    ]
    .iter()
    .any(|needle| lower.contains(needle));
    secret_key && (lower.contains('=') || lower.contains(':') || lower.contains("bearer "))
}

pub fn schema_summary(schema: &Value) -> Option<String> {
    let object = schema.as_object()?;
    let properties = object
        .get("properties")
        .and_then(Value::as_object)
        .map(|properties| properties.keys().take(8).cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    if properties.is_empty() {
        return None;
    }

    let required = object
        .get("required")
        .and_then(Value::as_array)
        .map(|required| {
            required
                .iter()
                .filter_map(Value::as_str)
                .take(8)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut summary = format!("fields: {}", properties.join(", "));
    if !required.is_empty() {
        summary.push_str("; required: ");
        summary.push_str(&required.join(", "));
    }
    Some(summary)
}
