#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchQuery {
    raw: String,
    limit: usize,
}

impl SearchQuery {
    pub fn new(raw: impl Into<String>) -> Self {
        Self {
            raw: raw.into(),
            limit: 10,
        }
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    pub fn normalized(&self) -> String {
        self.raw
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_ascii_lowercase()
    }

    pub fn limit_value(&self) -> usize {
        self.limit
    }
}
