use super::host_identity::HostIdentity;
use super::types::{ExtensionApiLevel, ExtensionUiManifest, ExtensionUiMode};
use serde_json::{json, Map, Value};

const OWNER: &str = "com.example.bounds";

fn manifest() -> ExtensionUiManifest {
    ExtensionUiManifest {
        api_version: "1".to_string(),
        mode: ExtensionUiMode::Standard,
        entry: None,
    }
}

fn validate(contribution: Value) -> Result<Vec<super::ui_types::UiCatalogEntry>, String> {
    super::ui_validation::catalog(
        &HostIdentity::ThirdParty(OWNER.to_string()),
        OWNER,
        &ExtensionApiLevel::Stable,
        Some(&manifest()),
        vec![contribution],
    )
}

fn localized(fill: &str) -> Value {
    let mut value = Map::new();
    value.insert("default".to_string(), json!(fill));
    for locale in super::ui_contract::UI_LOCALES {
        value.insert((*locale).to_string(), json!(fill));
    }
    Value::Object(value)
}

#[test]
fn view_nodes_depth_fields_and_options_accept_max_then_reject_max_plus_one() {
    let nodes = |count| {
        json!({
            "type":"settingsTab", "id":"nodes", "placement":"settings.navigation.preferences",
            "order":0, "label":{"default":"Nodes"},
            "detail":{"type":"stack","children":(1..count).map(|_| json!({
                "type":"text","text":{"default":"node"}
            })).collect::<Vec<_>>()}
        })
    };
    assert!(validate(nodes(super::ui_contract::MAX_VIEW_NODES)).is_ok());
    assert!(validate(nodes(super::ui_contract::MAX_VIEW_NODES + 1)).is_err());

    let depth = |count| {
        let mut node = json!({"type":"text","text":{"default":"leaf"}});
        for _ in 1..count {
            node = json!({"type":"stack","children":[node]});
        }
        json!({
            "type":"settingsTab", "id":"depth", "placement":"settings.navigation.preferences",
            "order":0, "label":{"default":"Depth"}, "detail":node
        })
    };
    assert!(validate(depth(super::ui_contract::MAX_VIEW_DEPTH)).is_ok());
    assert!(validate(depth(super::ui_contract::MAX_VIEW_DEPTH + 1)).is_err());

    let fields = |count| {
        json!({
            "type":"settingsTab", "id":"fields", "placement":"settings.navigation.preferences",
            "order":0, "label":{"default":"Fields"}, "detail":{"type":"stack","children":
                (0..count).map(|index| json!({
                    "type":"textField", "id":format!("field-{index}"),
                    "label":{"default":"Field"}, "value":""
                })).collect::<Vec<_>>()}
        })
    };
    assert!(validate(fields(super::ui_contract::MAX_FIELDS_PER_VIEW)).is_ok());
    assert!(validate(fields(super::ui_contract::MAX_FIELDS_PER_VIEW + 1)).is_err());

    let options = |count| {
        json!({
            "type":"settingsTab", "id":"options", "placement":"settings.navigation.preferences",
            "order":0, "label":{"default":"Options"}, "detail":{
                "type":"select", "id":"choice", "label":{"default":"Choice"}, "value":"",
                "options":(0..count).map(|index| json!({
                    "value":format!("choice-{index}"), "label":{"default":"Choice"}
                })).collect::<Vec<_>>()
            }
        })
    };
    assert!(validate(options(super::ui_contract::MAX_OPTIONS_PER_FIELD)).is_ok());
    assert!(validate(options(super::ui_contract::MAX_OPTIONS_PER_FIELD + 1)).is_err());
}

#[test]
fn text_and_extension_bytes_are_exact_while_theme_tokens_use_the_published_allowlist() {
    let text = |count| {
        json!({
            "type":"action", "id":"text", "placement":"app.toolbar.primary", "order":0,
            "label":{"default":"x".repeat(count)}, "actionId":"run"
        })
    };
    assert!(validate(text(super::ui_contract::MAX_TEXT_CHARS)).is_ok());
    assert!(validate(text(super::ui_contract::MAX_TEXT_CHARS + 1)).is_err());

    let theme = |count| {
        let mut tokens = Map::new();
        for name in super::ui_contract::UI_THEME_TOKENS.iter().take(count) {
            tokens.insert((*name).to_string(), json!("#112233"));
        }
        if count > super::ui_contract::UI_THEME_TOKENS.len() {
            tokens.insert("--unknown".to_string(), json!("#112233"));
        }
        json!({
            "type":"theme", "id":"theme", "order":0, "label":{"default":"Theme"},
            "base":"dark", "tokens":tokens
        })
    };
    assert!(super::ui_contract::UI_THEME_TOKENS.len() <= super::ui_contract::MAX_THEME_TOKENS);
    assert!(validate(theme(super::ui_contract::UI_THEME_TOKENS.len())).is_ok());
    assert!(validate(theme(super::ui_contract::UI_THEME_TOKENS.len() + 1)).is_err());

    let mut contributions = (0..17)
        .map(|index| {
            json!({
                "type":"action", "id":format!("{OWNER}.sized-{index}"),
                "placement":"app.toolbar.primary", "order":0,
                "label":localized("x"), "actionId":format!("{OWNER}.run-{index}")
            })
        })
        .collect::<Vec<_>>();
    fill_localized_strings_to_size(
        &mut contributions,
        super::ui_contract::MAX_UI_BYTES_PER_EXTENSION,
    );
    assert_eq!(
        serde_json::to_vec(&contributions).unwrap().len(),
        super::ui_contract::MAX_UI_BYTES_PER_EXTENSION
    );
    let identity = HostIdentity::ThirdParty(OWNER.to_string());
    assert!(super::ui_validation::catalog(
        &identity,
        OWNER,
        &ExtensionApiLevel::Stable,
        Some(&manifest()),
        contributions.clone()
    )
    .is_ok());
    grow_first_localized_string(&mut contributions);
    assert_eq!(
        serde_json::to_vec(&contributions).unwrap().len(),
        super::ui_contract::MAX_UI_BYTES_PER_EXTENSION + 1
    );
    assert!(super::ui_validation::catalog(
        &identity,
        OWNER,
        &ExtensionApiLevel::Stable,
        Some(&manifest()),
        contributions
    )
    .is_err());
}

#[test]
fn action_payload_and_result_bytes_accept_max_then_reject_max_plus_one() {
    let mut fields = Map::new();
    for index in 0..super::ui_contract::MAX_FIELDS_PER_VIEW {
        let prefix = format!("field-{index}-");
        let key = format!(
            "{prefix}{}",
            "x".repeat(super::types::MAX_IDENTIFIER_CHARS - prefix.len())
        );
        fields.insert(key, json!("x"));
    }
    let mut payload = json!({"fields": fields});
    fill_field_values_to_size(&mut payload, super::ui_contract::MAX_ACTION_PAYLOAD_BYTES);
    assert_eq!(
        serde_json::to_vec(&payload).unwrap().len(),
        super::ui_contract::MAX_ACTION_PAYLOAD_BYTES
    );
    let parsed: super::ui_types::UiActionPayload = serde_json::from_value(payload.clone()).unwrap();
    assert!(parsed.validate().is_ok());
    grow_first_field_value(&mut payload);
    let parsed: super::ui_types::UiActionPayload = serde_json::from_value(payload).unwrap();
    assert!(parsed.validate().is_err());

    let mut result = json!({
        "type":"view", "view":{"type":"stack","children":
            (0..17).map(|_| json!({"type":"text","text":localized("x")}))
                .collect::<Vec<_>>()
        }
    });
    fill_result_text_to_size(&mut result, super::ui_contract::MAX_ACTION_RESULT_BYTES);
    assert_eq!(
        serde_json::to_vec(&result).unwrap().len(),
        super::ui_contract::MAX_ACTION_RESULT_BYTES
    );
    assert!(super::ui_action_result::validate(OWNER, result.clone()).is_ok());
    grow_first_result_text(&mut result);
    assert_eq!(
        serde_json::to_vec(&result).unwrap().len(),
        super::ui_contract::MAX_ACTION_RESULT_BYTES + 1
    );
    assert!(super::ui_action_result::validate(OWNER, result).is_err());
}

fn fill_localized_strings_to_size(values: &mut [Value], target: usize) {
    let mut remaining = target
        .checked_sub(serde_json::to_vec(values).unwrap().len())
        .unwrap();
    for value in values {
        let label = value.get_mut("label").unwrap().as_object_mut().unwrap();
        for text in label.values_mut() {
            let current = text.as_str().unwrap();
            let room = super::ui_contract::MAX_TEXT_CHARS - current.chars().count();
            let added = room.min(remaining);
            *text = json!(format!("{current}{}", "x".repeat(added)));
            remaining -= added;
            if remaining == 0 {
                return;
            }
        }
    }
    assert_eq!(remaining, 0);
}

fn grow_first_localized_string(values: &mut [Value]) {
    for value in values {
        for text in value
            .get_mut("label")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .values_mut()
        {
            let current = text.as_str().unwrap();
            if current.chars().count() < super::ui_contract::MAX_TEXT_CHARS {
                *text = json!(format!("{current}x"));
                return;
            }
        }
    }
    panic!("fixture has no remaining character capacity");
}

fn fill_field_values_to_size(payload: &mut Value, target: usize) {
    let mut remaining = target
        .checked_sub(serde_json::to_vec(&payload).unwrap().len())
        .unwrap();
    for value in payload["fields"].as_object_mut().unwrap().values_mut() {
        let current = value.as_str().unwrap();
        let room = super::ui_contract::MAX_TEXT_CHARS - current.chars().count();
        let added = room.min(remaining);
        *value = json!(format!("{current}{}", "x".repeat(added)));
        remaining -= added;
        if remaining == 0 {
            return;
        }
    }
    assert_eq!(remaining, 0);
}

fn grow_first_field_value(payload: &mut Value) {
    for value in payload["fields"].as_object_mut().unwrap().values_mut() {
        let current = value.as_str().unwrap();
        if current.chars().count() < super::ui_contract::MAX_TEXT_CHARS {
            *value = json!(format!("{current}x"));
            return;
        }
    }
    panic!("payload fixture has no remaining character capacity");
}

fn fill_result_text_to_size(result: &mut Value, target: usize) {
    let mut remaining = target
        .checked_sub(serde_json::to_vec(&result).unwrap().len())
        .unwrap();
    for node in result["view"]["children"].as_array_mut().unwrap() {
        for text in node["text"].as_object_mut().unwrap().values_mut() {
            let current = text.as_str().unwrap();
            let room = super::ui_contract::MAX_TEXT_CHARS - current.chars().count();
            let added = room.min(remaining);
            *text = json!(format!("{current}{}", "x".repeat(added)));
            remaining -= added;
            if remaining == 0 {
                return;
            }
        }
    }
    assert_eq!(remaining, 0);
}

fn grow_first_result_text(result: &mut Value) {
    for node in result["view"]["children"].as_array_mut().unwrap() {
        for text in node["text"].as_object_mut().unwrap().values_mut() {
            let current = text.as_str().unwrap();
            if current.chars().count() < super::ui_contract::MAX_TEXT_CHARS {
                *text = json!(format!("{current}x"));
                return;
            }
        }
    }
    panic!("result fixture has no remaining character capacity");
}
