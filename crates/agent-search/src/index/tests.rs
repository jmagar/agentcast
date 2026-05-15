use super::*;
use agent_protocol::LauncherActionId;

fn doc(id: &str, name: &str, description: Option<&str>) -> SearchDocument {
    SearchDocument {
        action_id: LauncherActionId::new(id),
        name: name.to_string(),
        description: description.map(str::to_string),
        metadata: vec!["safe metadata".to_string()],
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
        catalog_hash: "hash-a".to_string(),
        truncated: true,
    }]);

    let results = index.search(SearchQuery::new("git porcelain").limit(5));

    assert_eq!(results[0].match_kind, SearchMatchKind::Token);
    assert!(results[0].truncated);
}
