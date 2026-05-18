use super::*;

#[test]
fn parses_package_and_dependency_sections() {
    let manifest = parse_manifest_dependencies(
        r#"
        [package]
        name = "agent-api"

        [dependencies]
        agent-mcp = { path = "../agent-mcp" }

        [dev-dependencies]
        agent-core = { path = "../agent-core" }
        "#,
    )
    .unwrap()
    .unwrap();

    assert_eq!(manifest.package, "agent-api");
    assert!(manifest.has_dependency("agent-mcp"));
    assert!(manifest.has_dependency("agent-core"));
}

#[test]
fn skips_workspace_root_manifest_without_package() {
    let manifest = parse_manifest_dependencies(
        r#"
        [workspace]
        members = ["crates/*"]
        "#,
    )
    .unwrap();

    assert_eq!(manifest, None);
}

#[test]
fn flags_surface_to_adapter_dependency() {
    let manifest = ManifestDeps {
        package: "agent-api".to_owned(),
        dependencies: vec!["agent-mcp".to_owned()],
    };
    let mut findings = Vec::new();

    check_manifest(&manifest, &mut findings);

    assert_eq!(findings.len(), 1);
    assert!(findings[0].contains("agent-api"));
    assert!(findings[0].contains("agent-mcp"));
}

#[test]
fn allows_documented_agent_server_stdio_gateway_exception() {
    let manifest = ManifestDeps {
        package: "agent-server".to_owned(),
        dependencies: vec!["agent-mcp".to_owned()],
    };
    let mut findings = Vec::new();

    check_manifest(&manifest, &mut findings);

    assert!(findings.is_empty());
}
