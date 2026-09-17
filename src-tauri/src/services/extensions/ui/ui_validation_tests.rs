use super::host_identity::HostIdentity;
use super::types::{ExtensionApiLevel, ExtensionUiManifest, ExtensionUiMode};
use serde_json::{json, Value};

fn manifest() -> ExtensionUiManifest {
    ExtensionUiManifest {
        api_version: "1".to_string(),
        mode: ExtensionUiMode::Standard,
        entry: None,
    }
}

fn action(id: &str) -> Value {
    json!({
        "type":"action", "id":id, "placement":"app.toolbar.primary", "order":0,
        "label":{"default":"Run","zh":"运行"}, "icon":"sparkle", "actionId":id
    })
}

#[test]
fn third_party_identity_and_standard_manifest_are_authoritative() {
    let identity = HostIdentity::ThirdParty("com.example.owner".to_string());
    assert!(super::ui_validation::catalog(
        &identity,
        "com.example.owner",
        &ExtensionApiLevel::Stable,
        Some(&manifest()),
        vec![action("run")],
    )
    .is_ok());
    assert!(super::ui_validation::catalog(
        &identity,
        "com.example.other",
        &ExtensionApiLevel::Stable,
        Some(&manifest()),
        vec![action("run")],
    )
    .is_err());
}

#[test]
fn advanced_manifest_stays_inert_until_its_dedicated_host_lot() {
    let identity = HostIdentity::ThirdParty("com.example.owner".to_string());
    let advanced = ExtensionUiManifest {
        api_version: "1".to_string(),
        mode: ExtensionUiMode::Advanced,
        entry: Some("ui/index.mjs".to_string()),
    };
    assert_eq!(
        super::ui_validation::catalog(
            &identity,
            "com.example.owner",
            &ExtensionApiLevel::Advanced,
            Some(&advanced),
            Vec::new(),
        )
        .unwrap(),
        Vec::new(),
    );
    assert!(super::ui_validation::catalog(
        &identity,
        "com.example.owner",
        &ExtensionApiLevel::Advanced,
        Some(&advanced),
        vec![action("run")],
    )
    .is_err());
}

#[test]
fn contribution_limit_accepts_max_and_rejects_max_plus_one() {
    let identity = HostIdentity::ThirdParty("com.example.owner".to_string());
    let make = |count| {
        (0..count)
            .map(|index| action(&format!("run-{index}")))
            .collect::<Vec<_>>()
    };
    assert!(super::ui_validation::catalog(
        &identity,
        "com.example.owner",
        &ExtensionApiLevel::Stable,
        Some(&manifest()),
        make(super::ui_contract::MAX_CONTRIBUTIONS_PER_EXTENSION),
    )
    .is_ok());
    assert!(super::ui_validation::catalog(
        &identity,
        "com.example.owner",
        &ExtensionApiLevel::Stable,
        Some(&manifest()),
        make(super::ui_contract::MAX_CONTRIBUTIONS_PER_EXTENSION + 1),
    )
    .is_err());
}

#[test]
fn nested_button_actions_and_cjk_are_collected_and_depth_is_bounded() {
    let identity = HostIdentity::ThirdParty("com.example.owner".to_string());
    let contribution = json!({
        "type":"settingsTab", "id":"panel",
        "placement":"settings.navigation.preferences", "order":1,
        "label":{"default":"設定","ja":"設定"},
        "detail":{"type":"stack","children":[{
            "type":"button","id":"apply","label":{"default":"保存"},"actionId":"apply"
        }]}
    });
    let entries = super::ui_validation::catalog(
        &identity,
        "com.example.owner",
        &ExtensionApiLevel::Stable,
        Some(&manifest()),
        vec![contribution],
    )
    .unwrap();
    assert_eq!(entries[0].action_ids, ["com.example.owner.apply"]);

    let mut node = json!({"type":"text","text":{"default":"leaf"}});
    for _ in 0..super::ui_contract::MAX_VIEW_DEPTH {
        node = json!({"type":"stack","children":[node]});
    }
    let too_deep = json!({
        "type":"tab", "id":"deep", "placement":"app.navigation.primary", "order":0,
        "label":{"default":"Deep"}, "detail":node
    });
    assert!(super::ui_validation::catalog(
        &identity,
        "com.example.owner",
        &ExtensionApiLevel::Stable,
        Some(&manifest()),
        vec![too_deep],
    )
    .is_err());
}

#[test]
fn duplicate_action_ids_and_oversized_text_fail_closed() {
    let identity = HostIdentity::ThirdParty("com.example.owner".to_string());
    let mut too_large = action("large");
    too_large["label"]["default"] =
        Value::String("x".repeat(super::ui_contract::MAX_TEXT_CHARS + 1));
    assert!(super::ui_validation::catalog(
        &identity,
        "com.example.owner",
        &ExtensionApiLevel::Stable,
        Some(&manifest()),
        vec![action("same"), action("same")],
    )
    .is_err());
    assert!(super::ui_validation::catalog(
        &identity,
        "com.example.owner",
        &ExtensionApiLevel::Stable,
        Some(&manifest()),
        vec![too_large],
    )
    .is_err());
}
