use super::*;
use crate::schema_summary;
use agent_protocol::LauncherActionId;

fn doc(id: &str, name: &str, description: Option<&str>) -> SearchDocument {
    SearchDocument {
        action_id: LauncherActionId::new(id),
        name: name.to_string(),
        description: description.map(str::to_string),
        metadata: vec!["safe metadata".to_string()],
        schema_summary: None,
        catalog_hash: "hash-a".to_string(),
        truncated: false,
    }
}

#[test]
fn exact_action_name_ranks_first() {
    let index = SearchIndex::new(vec![
        doc("mcp:git:status-long", "Git status long", Some("git status")),
        doc(
            "mcp:git:status",
            "Git status",
            Some("inspect repository state"),
        ),
    ]);

    let results = index.search(SearchQuery::new("git status").limit(10));

    assert_eq!(results[0].action_id.as_str(), "mcp:git:status");
    assert_eq!(results[0].match_kind, SearchMatchKind::ExactName);
}

#[test]
fn empty_query_returns_no_results() {
    let index = SearchIndex::new(vec![doc("mcp:git:status", "Git status", None)]);

    assert!(index.search(SearchQuery::new("   ")).is_empty());
}

#[test]
fn top_k_and_tie_breaks_are_deterministic() {
    let index = SearchIndex::new(vec![
        doc("mcp:z:match", "Match Z", None),
        doc("mcp:a:match", "Match A", None),
    ]);

    let results = index.search(SearchQuery::new("match").limit(1));

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].action_id.as_str(), "mcp:a:match");
}

#[test]
fn token_match_searches_description_and_metadata() {
    let index = SearchIndex::new(vec![SearchDocument {
        action_id: LauncherActionId::new("mcp:git:status"),
        name: "Repository state".to_string(),
        description: Some("Inspect git working tree".to_string()),
        metadata: vec!["status porcelain".to_string()],
        schema_summary: None,
        catalog_hash: "hash-a".to_string(),
        truncated: true,
    }]);

    let results = index.search(SearchQuery::new("git porcelain").limit(5));

    assert_eq!(results[0].match_kind, SearchMatchKind::Token);
    assert!(results[0].truncated);
    assert_eq!(results[0].matched_terms, vec!["git", "porcelain"]);
    assert_eq!(
        results[0].matched_fields,
        vec![SearchMatchField::Description, SearchMatchField::Metadata]
    );
}

#[test]
fn search_index_redacts_secret_like_metadata_before_matching() {
    let index = SearchIndex::new(vec![SearchDocument {
        action_id: LauncherActionId::new("mcp:secret:read"),
        name: "Secret reader".to_string(),
        description: None,
        metadata: vec!["API_TOKEN=super-secret-value".to_string()],
        schema_summary: None,
        catalog_hash: "hash-a".to_string(),
        truncated: false,
    }]);

    assert!(
        index
            .search(SearchQuery::new("super-secret-value"))
            .is_empty()
    );
    let results = index.search(SearchQuery::new("redacted").limit(5));

    assert_eq!(results[0].action_id.as_str(), "mcp:secret:read");
    assert!(results[0].truncated);
    assert_eq!(results[0].matched_fields, vec![SearchMatchField::Metadata]);
}

#[test]
fn schema_summary_is_searchable_and_reported_as_match_field() {
    let index = SearchIndex::new(vec![SearchDocument {
        action_id: LauncherActionId::new("mcp:files:write"),
        name: "Write file".to_string(),
        description: None,
        metadata: Vec::new(),
        schema_summary: Some("requires path and content fields".to_string()),
        catalog_hash: "hash-a".to_string(),
        truncated: false,
    }]);

    let results = index.search(SearchQuery::new("content fields").limit(5));

    assert_eq!(
        results[0].summary.as_deref(),
        Some("requires path and content fields")
    );
    assert_eq!(
        results[0].matched_fields,
        vec![SearchMatchField::SchemaSummary]
    );
}

#[test]
fn long_fields_are_truncated_before_result_summary() {
    let index = SearchIndex::new(vec![SearchDocument {
        action_id: LauncherActionId::new("mcp:long:tool"),
        name: "Long tool".to_string(),
        description: Some("x".repeat(300)),
        metadata: Vec::new(),
        schema_summary: None,
        catalog_hash: "hash-a".to_string(),
        truncated: false,
    }]);

    let results = index.search(SearchQuery::new("long").limit(5));

    assert!(results[0].truncated);
    assert!(
        results[0]
            .summary
            .as_ref()
            .expect("summary")
            .ends_with("...")
    );
}

#[test]
fn summarizes_object_schema_fields_and_required_keys() {
    let summary = schema_summary(&serde_json::json!({
        "type": "object",
        "properties": {
            "path": { "type": "string" },
            "content": { "type": "string" }
        },
        "required": ["path"]
    }))
    .expect("summary");

    assert_eq!(summary, "fields: content, path; required: path");
}
