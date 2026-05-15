use super::*;

#[test]
fn parses_space_separated_scopes_in_sorted_order() {
    let scopes = ScopeSet::parse("mcp:write mcp:read mcp:read").expect("parse scopes");

    assert_eq!(scopes.as_slice(), &["mcp:read", "mcp:write"]);
}

#[test]
fn rejects_empty_scope_segments() {
    let error = ScopeSet::parse("mcp:read  ").expect_err("invalid scopes");

    assert_eq!(error.to_string(), "scope string contains an empty segment");
}

#[test]
fn required_scopes_must_all_be_present() {
    let granted = ScopeSet::parse("mcp:read mcp:write").expect("parse granted");
    let required = ScopeSet::parse("mcp:read").expect("parse required");
    let missing = ScopeSet::parse("mcp:admin").expect("parse missing");

    assert!(granted.contains_all(&required));
    assert!(!granted.contains_all(&missing));
}
