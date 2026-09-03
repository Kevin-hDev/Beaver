use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::path::Path;

const UI_BOOTSTRAP_MAX_BYTES: usize = 256;
const UI_CONTRACT_MAX_BYTES: usize = 1_048_576;

fn contract() -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    serde_json::from_slice(
        &std::fs::read(root.join("resources/extension-ui/contract.json")).unwrap(),
    )
    .unwrap()
}

#[test]
fn ui_contract_declares_the_complete_initial_surface() {
    let contract = contract();

    assert_eq!(contract["apiVersion"], "1");
    assert_eq!(contract["modes"], json!(["standard", "advanced"]));
    assert_eq!(
        contract["contributionTypes"],
        json!(["tab", "settingsTab", "action", "theme"])
    );
    assert_eq!(
        contract["primitives"],
        json!([
            "stack",
            "row",
            "heading",
            "text",
            "badge",
            "separator",
            "textField",
            "numberField",
            "select",
            "toggle",
            "button"
        ])
    );
    assert_eq!(contract["themeBases"], json!(["light", "dark"]));
    assert_eq!(
        contract["locales"],
        json!(["fr", "en", "es", "de", "it", "zh", "ja"])
    );
}

#[test]
fn ui_contract_declares_public_placements_and_protected_occupants_once() {
    let contract = contract();
    let placements = contract["placements"].as_array().unwrap();
    let keys = placements
        .iter()
        .map(|placement| placement["key"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(
        keys,
        [
            "app.navigation.primary",
            "settings.navigation.preferences",
            "settings.navigation.agent",
            "settings.navigation.models",
            "settings.navigation.integrations",
            "settings.navigation.application",
            "app.toolbar.primary",
            "agent.composer.leading",
        ]
    );
    assert_eq!(keys.iter().collect::<BTreeSet<_>>().len(), keys.len());
    assert_eq!(
        contract["protectedOccupants"],
        json!([
            {
                "placement": "app.navigation.primary",
                "occupant": "beaver.settings",
                "operations": ["remove", "replace"]
            },
            {
                "placement": "settings.navigation.integrations",
                "occupant": "beaver.extensions",
                "operations": ["remove", "replace"]
            }
        ])
    );
}

#[test]
fn ui_contract_collections_and_numeric_limits_are_bounded() {
    let contract = contract();
    let maximum = contract["validation"]["maxNumericLimit"].as_u64().unwrap();
    let limits = contract["limits"].as_object().unwrap();

    assert_eq!(maximum, 4_194_304);
    assert!(limits.values().all(|value| value
        .as_u64()
        .is_some_and(|value| value > 0 && value <= maximum)));
    assert_eq!(limits["maxContributionsPerExtension"], 32);
    assert_eq!(limits["maxGlobalStandardContributions"], 512);
    assert_eq!(limits["maxAdvancedArtifactBytes"], 4_194_304);
    assert_eq!(limits["maxAdvancedArtifactFiles"], 64);
    assert_eq!(limits["maxAdvancedMountsPerExtension"], 32);

    let theoretical = limits["maxContributionsPerExtension"]
        .as_u64()
        .unwrap()
        .checked_mul(128)
        .unwrap();
    assert!(limits["maxGlobalStandardContributions"].as_u64().unwrap() <= theoretical);
    assert!(
        limits["maxGlobalUiBytes"].as_u64().unwrap()
            <= limits["maxUiBytesPerExtension"]
                .as_u64()
                .unwrap()
                .checked_mul(128)
                .unwrap()
    );
}

#[test]
fn every_public_theme_token_exists_in_each_shipped_theme() {
    let contract = contract();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/styles/themes");

    for theme in [
        "astral-mist.css",
        "cobalt-frost.css",
        "crimson-eclipse.css",
        "dark.css",
        "emerald-night.css",
        "light.css",
    ] {
        let source = std::fs::read_to_string(root.join(theme)).unwrap();
        for token in contract["themeTokens"].as_array().unwrap() {
            let declaration = format!("{}:", token.as_str().unwrap());
            assert!(
                source.contains(&declaration),
                "{theme} misses {declaration}"
            );
        }
    }
}

#[test]
fn ui_contract_names_loading_stages_diagnostics_icons_and_theme_tokens() {
    let contract = contract();

    for name in ["icons", "themeTokens", "loadingStages", "diagnosticCodes"] {
        let values = contract[name].as_array().unwrap();
        assert!(!values.is_empty(), "{name} must not be empty");
        let unique = values
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(unique.len(), values.len(), "{name} contains duplicates");
    }
    assert_eq!(
        contract["loadingStages"],
        json!(["contract", "bundle", "approve", "import", "activate", "mount"])
    );
    assert!(contract["diagnosticCodes"]
        .as_array()
        .unwrap()
        .contains(&json!("ui_protocol_request_denied")));
    assert!(contract["themeTokens"]
        .as_array()
        .unwrap()
        .contains(&json!("--diff-add-bg")));
}

#[test]
fn ui_contract_bootstrap_and_contract_files_are_bounded() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/extension-ui");
    let bootstrap = std::fs::read(root.join("contract-bootstrap.json")).unwrap();
    let contract = std::fs::read(root.join("contract.json")).unwrap();
    let bootstrap_json: Value = serde_json::from_slice(&bootstrap).unwrap();

    assert!(bootstrap.len() <= UI_BOOTSTRAP_MAX_BYTES);
    assert_eq!(
        bootstrap_json,
        json!({ "maxContractBytes": UI_CONTRACT_MAX_BYTES })
    );
    assert!(contract.len() <= UI_CONTRACT_MAX_BYTES);
}

#[test]
fn host_contract_exposes_ui_action_in_only_the_core_to_host_direction() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let host: Value = serde_json::from_slice(
        &std::fs::read(root.join("resources/extension-host/contract.json")).unwrap(),
    )
    .unwrap();

    assert!(host["methods"]["coreToHost"]
        .as_array()
        .unwrap()
        .contains(&json!("ui.action")));
    assert!(!host["methods"]["hostToCore"]
        .as_array()
        .unwrap()
        .iter()
        .any(|method| method["name"] == "ui.action"));
}
