use super::*;

#[test]
fn error_kind_strings_are_stable() {
    let error = ErrorInfo::new(CoreErrorKind::Conflict, "already exists");
    assert_eq!(error.kind_str(), "conflict");
    assert_eq!(serde_json::to_value(error).unwrap()["kind"], "conflict");
}
