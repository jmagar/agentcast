use super::*;

#[test]
fn oauth_key_requires_32_decoded_bytes() {
    assert!(parse_oauth_key(&"07".repeat(32)).is_ok());
    assert!(parse_oauth_key("07").is_err());
}

#[test]
fn oauth_store_flags_must_be_complete() {
    let path = std::path::PathBuf::from("oauth.db");
    assert!(oauth_router_for_store_args(Some(&path), None).is_err());

    let key = "07".repeat(32);
    assert!(oauth_router_for_store_args(None, Some(&key)).is_err());
}
