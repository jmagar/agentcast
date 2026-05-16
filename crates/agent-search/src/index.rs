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
    pub matched_fields: Vec<SearchMatchField>,
    pub matched_terms: Vec<String>,
    pub summary: Option<String>,
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchMatchField {
    Name,
    Description,
    Metadata,
    SchemaSummary,
}

#[derive(Clone, Debug, Default)]
pub struct SearchIndex {
    documents: Vec<SearchDocument>,
}

impl SearchIndex {
    pub fn new(documents: Vec<SearchDocument>) -> Self {
        Self {
            documents: documents
                .into_iter()
                .map(SearchDocument::sanitized)
                .collect(),
        }
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
    let (score, match_kind, matched_terms) = if name == normalized_query {
        (300, SearchMatchKind::ExactName)
    } else if name.contains(normalized_query) {
        (200, SearchMatchKind::Substring)
    } else if tokens.iter().all(|token| text.contains(token)) {
        (100 + tokens.len() as u16, SearchMatchKind::Token)
    } else {
        return None;
    }
    .with_terms(normalized_query, tokens);
    let matched_fields = matched_fields(document, &matched_terms);

    Some(SearchResult {
        action_id: document.action_id.clone(),
        name: document.name.clone(),
        score,
        match_kind,
        matched_fields,
        matched_terms,
        summary: document
            .description
            .clone()
            .or_else(|| document.schema_summary.clone()),
        catalog_hash: document.catalog_hash.clone(),
        truncated: document.truncated,
    })
}

trait MatchTerms {
    fn with_terms(
        self,
        normalized_query: &str,
        tokens: &[&str],
    ) -> (u16, SearchMatchKind, Vec<String>);
}

impl MatchTerms for (u16, SearchMatchKind) {
    fn with_terms(
        self,
        normalized_query: &str,
        tokens: &[&str],
    ) -> (u16, SearchMatchKind, Vec<String>) {
        let terms = match self.1 {
            SearchMatchKind::ExactName | SearchMatchKind::Substring => {
                vec![normalized_query.to_string()]
            }
            SearchMatchKind::Token => tokens.iter().map(|token| (*token).to_string()).collect(),
        };
        (self.0, self.1, terms)
    }
}

fn matched_fields(document: &SearchDocument, terms: &[String]) -> Vec<SearchMatchField> {
    let mut fields = Vec::new();
    push_if_contains(
        &mut fields,
        SearchMatchField::Name,
        std::slice::from_ref(&document.name),
        terms,
    );
    if let Some(description) = &document.description {
        push_if_contains(
            &mut fields,
            SearchMatchField::Description,
            std::slice::from_ref(description),
            terms,
        );
    }
    push_if_contains(
        &mut fields,
        SearchMatchField::Metadata,
        &document.metadata,
        terms,
    );
    if let Some(summary) = &document.schema_summary {
        push_if_contains(
            &mut fields,
            SearchMatchField::SchemaSummary,
            std::slice::from_ref(summary),
            terms,
        );
    }
    fields
}

fn push_if_contains(
    fields: &mut Vec<SearchMatchField>,
    field: SearchMatchField,
    values: &[String],
    terms: &[String],
) {
    if values.iter().any(|value| {
        let value = value.to_ascii_lowercase();
        terms.iter().any(|term| value.contains(term))
    }) {
        fields.push(field);
    }
}
