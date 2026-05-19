use super::*;

#[test]
fn redacts_secret_like_keys_only() {
    assert_eq!(redact_key_value("API_TOKEN", "abc"), REDACTED);
    assert_eq!(redact_key_value("name", "filesystem"), "filesystem");
    assert!(should_redact_key("authorization"));
}
