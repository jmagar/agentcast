use super::*;

#[test]
fn permission_option_kind_has_stable_json() {
    let option = PermissionOption {
        id: "allow_once".into(),
        label: "Allow once".into(),
        kind: PermissionOptionKind::AllowOnce,
    };
    assert_eq!(serde_json::to_value(option).unwrap()["kind"], "allow_once");
}
