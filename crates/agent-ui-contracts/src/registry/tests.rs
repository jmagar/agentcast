use super::*;

#[test]
fn registry_view_omits_missing_optionals() {
    let value = serde_json::to_value(RegistryServerView {
        name: "io.modelcontextprotocol/filesystem".into(),
        description: None,
        latest_version: None,
        package_count: 1,
    })
    .unwrap();
    assert_eq!(value["package_count"], 1);
}
