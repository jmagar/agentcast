use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StableId(String);

impl StableId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub fn stable_id(parts: impl IntoIterator<Item = impl AsRef<str>>) -> StableId {
    StableId::new(
        parts
            .into_iter()
            .map(|part| {
                part.as_ref()
                    .chars()
                    .map(|ch| {
                        if ch.is_ascii_alphanumeric() {
                            ch.to_ascii_lowercase()
                        } else {
                            '_'
                        }
                    })
                    .collect::<String>()
                    .trim_matches('_')
                    .to_string()
            })
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(":"),
    )
}
