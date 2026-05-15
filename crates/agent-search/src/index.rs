use crate::{SearchDocument, SearchQuery};
use agent_protocol::LauncherActionId;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchResult {
    pub action_id: LauncherActionId,
    pub name: String,
    pub score: u16,
    pub match_kind: SearchMatchKind,
    pub catalog_hash: String,
    pub truncated: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchMatchKind {
    ExactName,
    Substring,
    Token,
}

#[derive(Clone, Debug, Default)]
pub struct SearchIndex {
    documents: Vec<SearchDocument>,
}

impl SearchIndex {
    pub fn new(documents: Vec<SearchDocument>) -> Self {
        Self { documents }
    }

    pub fn search(&self, query: SearchQuery) -> Vec<SearchResult> {
        let normalized = query.normalized();
        if normalized.is_empty() || query.limit_value() == 0 {
            return Vec::new();
        }

        let tokens = normalized.split(' ').collect::<Vec<_>>();
        let mut results = self
            .documents
            .iter()
            .filter_map(|document| score_document(document, &normalized, &tokens))
            .collect::<Vec<_>>();

        results.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.name.cmp(&right.name))
                .then_with(|| left.action_id.cmp(&right.action_id))
        });
        results.truncate(query.limit_value());
        results
    }
}

fn score_document(
    document: &SearchDocument,
    normalized_query: &str,
    tokens: &[&str],
) -> Option<SearchResult> {
    let name = document.name.to_ascii_lowercase();
    let text = document.searchable_text().to_ascii_lowercase();
    let (score, match_kind) = if name == normalized_query {
        (300, SearchMatchKind::ExactName)
    } else if name.contains(normalized_query) {
        (200, SearchMatchKind::Substring)
    } else if tokens.iter().all(|token| text.contains(token)) {
        (100 + tokens.len() as u16, SearchMatchKind::Token)
    } else {
        return None;
    };

    Some(SearchResult {
        action_id: document.action_id.clone(),
        name: document.name.clone(),
        score,
        match_kind,
        catalog_hash: document.catalog_hash.clone(),
        truncated: document.truncated,
    })
}
