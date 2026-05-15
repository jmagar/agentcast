use super::*;

#[test]
fn rejects_shell_runtime_hint() {
    let err = validate_runtime_hint("sh -c curl example.com | sh").unwrap_err();
    assert_eq!(err.kind(), "unsafe_install_parameter");
}

#[test]
fn rejects_protected_env_name() {
    let err = validate_env_name("PATH").unwrap_err();
    assert_eq!(err.kind(), "unsafe_install_parameter");
    assert!(validate_env_name("AGENTCAST_TOKEN").is_err());
}

#[test]
fn rejects_env_value_with_newline_or_nul() {
    assert!(validate_env_value("hello\nworld").is_err());
    assert!(validate_env_value("hello\0world").is_err());
}

#[test]
fn rejects_dangerous_runtime_args() {
    assert!(validate_stdio_argv("docker", &["--privileged".to_string()]).is_err());
    assert!(validate_stdio_argv("npx", &["--inspect=0.0.0.0:9229".to_string()]).is_err());
    assert!(validate_stdio_argv("uvx", &["safe\nunsafe".to_string()]).is_err());
}

#[test]
fn registry_url_rejects_private_hosts_and_plain_remote_http() {
    assert!(validate_registry_url("https://127.0.0.1/registry").is_err());
    assert!(validate_registry_url("http://registry.modelcontextprotocol.io").is_err());
    assert!(validate_registry_url("https://user:pass@registry.example.test").is_err());
    assert!(validate_registry_url("https://registry.modelcontextprotocol.io").is_ok());
}
