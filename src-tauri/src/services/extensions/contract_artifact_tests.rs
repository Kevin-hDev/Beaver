#[path = "../../../extension_contract_shared.rs"]
mod extension_contract_shared;
#[path = "../../../extension_contract_build.rs"]
#[allow(dead_code)]
mod generator;
#[path = "../../../extension_ui_contract_build.rs"]
#[allow(dead_code)]
mod ui_generator;

const UI_GENERATED_BEGIN: &str = "<!-- BEGIN GENERATED EXTENSION UI CONTRACT -->";
const UI_GENERATED_END: &str = "<!-- END GENERATED EXTENSION UI CONTRACT -->";

#[test]
fn oversized_contract_is_rejected_before_deserialization() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("contract-bootstrap.json"),
        br#"{"maxContractBytes":8192}"#,
    )
    .unwrap();
    std::fs::write(directory.path().join("contract.json"), vec![b' '; 8_193]).unwrap();

    let error = generator::load_contract(directory.path()).unwrap_err();

    assert_eq!(error, "Beaver extension contract exceeds its size limit");
}

#[test]
fn bootstrap_is_bounded_and_controls_the_rust_contract_reader() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("contract-bootstrap.json"),
        vec![b' '; generator::BOOTSTRAP_FILE_MAX_BYTES + 1],
    )
    .unwrap();
    std::fs::write(directory.path().join("contract.json"), b"{}").unwrap();
    assert_eq!(
        generator::load_contract(directory.path()).unwrap_err(),
        "Beaver extension contract bootstrap exceeds its size limit"
    );

    std::fs::write(
        directory.path().join("contract-bootstrap.json"),
        br#"{"maxContractBytes":4}"#,
    )
    .unwrap();
    std::fs::write(directory.path().join("contract.json"), b"{\"x\":1}").unwrap();
    assert_eq!(
        generator::load_contract(directory.path()).unwrap_err(),
        "Beaver extension contract exceeds its size limit"
    );
}

#[test]
fn bootstrap_rejects_extra_keys_and_values_outside_its_fixed_range() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("contract.json"), b"{}").unwrap();
    for bootstrap in [
        br#"{"maxContractBytes":0}"#.as_slice(),
        br#"{"maxContractBytes":1048577}"#.as_slice(),
        br#"{"maxContractBytes":8192,"extra":1}"#.as_slice(),
    ] {
        std::fs::write(directory.path().join("contract-bootstrap.json"), bootstrap).unwrap();
        assert_eq!(
            generator::load_contract(directory.path()).unwrap_err(),
            "invalid Beaver extension contract bootstrap"
        );
    }
}

#[test]
fn contract_declares_the_complete_v1_surface() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let contract = generator::load_contract(&root.join("resources/extension-host")).unwrap();

    assert_eq!(contract["apiVersion"], "1");
    assert_eq!(
        contract["capabilities"],
        serde_json::json!(["tools", "events", "ui"])
    );
    assert_eq!(
        contract["methods"]["coreToHost"],
        serde_json::json!([
            "host.hello",
            "host.reset",
            "host.load",
            "tool.call",
            "event.emit",
            "ui.action"
        ])
    );
    assert_eq!(
        contract["events"],
        serde_json::json!(["session.turn.started"])
    );
    assert_eq!(
        contract["effectClasses"],
        serde_json::json!([
            "read-only",
            "local-write",
            "external-read",
            "external-write",
            "process",
            "secret",
            "unknown"
        ])
    );
    assert_eq!(contract["validation"]["maxNumericLimit"], 1_073_741_824_u64);
    assert_eq!(
        contract["limits"]["fingerprintMaxTotalBytes"],
        33_554_432_u64
    );
}

#[test]
fn contract_validates_shared_limits_timeouts_and_builtin_count() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let directory = root.join("resources/extension-host");
    let contract = generator::load_contract(&directory).unwrap();

    generator::validate_contract(&contract, &directory).unwrap();
    assert_eq!(
        contract["limits"]["maxHostProcesses"].as_u64().unwrap()
            + contract["limits"]["minLongLivedAppWorkReserve"]
                .as_u64()
                .unwrap(),
        crate::app_exit::REGISTRY_CAPACITY as u64
    );
    assert_eq!(contract["limits"]["maxIdentifierChars"], 96);
}

#[test]
fn contract_rejects_names_that_the_node_host_would_reject() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let directory = root.join("resources/extension-host");
    let contract = generator::load_contract(&directory).unwrap();
    let mut invalid = contract.clone();
    invalid["capabilities"][0] = serde_json::json!("Tools");

    assert!(generator::validate_contract(&invalid, &directory).is_err());
}

#[test]
fn contract_rejects_numeric_and_timeout_values_outside_shared_rules() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let directory = root.join("resources/extension-host");
    let contract = generator::load_contract(&directory).unwrap();

    let mut numeric = contract.clone();
    numeric["limits"]["fingerprintMaxTotalBytes"] = serde_json::json!(1_073_741_825_u64);
    assert!(generator::validate_contract(&numeric, &directory).is_err());

    let mut tool_timeout = contract.clone();
    tool_timeout["timeouts"]["toolCallTimeoutMs"] =
        tool_timeout["timeouts"]["hostRequestTimeoutMs"].clone();
    assert!(generator::validate_contract(&tool_timeout, &directory).is_err());

    let mut mcp_timeout = contract;
    mcp_timeout["timeouts"]["mcpToolTimeoutMs"] =
        mcp_timeout["timeouts"]["coreRequestTimeoutMs"].clone();
    assert!(generator::validate_contract(&mcp_timeout, &directory).is_err());
}

#[test]
fn checked_in_typescript_matches_the_extension_contract() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let directory = root.join("resources/extension-host");
    let contract = generator::load_contract(&directory).unwrap();
    let checked_in =
        include_str!("../../../../src/types/extension-contract.generated.ts").replace("\r\n", "\n");

    assert_eq!(checked_in, generator::render_typescript(&contract).unwrap());
    assert!(checked_in.contains("export const EXTENSION_HOST_STATES"));
    assert!(checked_in.contains("export type ExtensionHostState"));
    assert!(checked_in.contains("export const HOST_DIAGNOSTIC_CODES"));
    assert!(checked_in.contains("export const RUNTIME_DIAGNOSTIC_CODES"));
}

#[test]
fn checked_in_sdk_contract_matches_the_extension_contract() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let directory = root.join("resources/extension-host");
    let contract = generator::load_contract(&directory).unwrap();
    let checked_in =
        include_str!("../../../resources/extension-host/sdk/contract.d.ts").replace("\r\n", "\n");

    assert_eq!(
        checked_in,
        generator::render_sdk_contract(&contract).unwrap()
    );
    assert!(checked_in.contains("STABLE_HOST_TO_CORE_REQUEST_METHODS: readonly [\"app.info\""));
    assert!(!checked_in
        .lines()
        .find(|line| line.contains("STABLE_HOST_TO_CORE_REQUEST_METHODS"))
        .unwrap()
        .contains("host.load.stage"));
    assert!(
        checked_in.contains("HOST_TO_CORE_NOTIFICATION_METHODS: readonly [\"host.load.stage\"]")
    );
}

#[test]
fn generated_rust_names_the_unique_load_stage_notification() {
    let generated = include_str!(concat!(env!("OUT_DIR"), "/extension_contract.rs"));

    assert!(generated.contains("pub const HOST_LOAD_STAGE_METHOD: &str = \"host.load.stage\";"));
    assert!(generated.contains("pub enum HostState"));
}

#[test]
fn ui_contract_rejects_unknown_schema_keys_at_every_structured_level() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let contract = ui_generator::load_contract(&root.join("resources/extension-ui")).unwrap();

    let mut top_level = contract.clone();
    top_level
        .as_object_mut()
        .unwrap()
        .insert("futureSection".to_string(), serde_json::json!({}));
    assert!(ui_generator::validate_contract(&top_level).is_err());

    for pointer in ["/placements/0", "/protectedOccupants/0", "/validation"] {
        let mut mutated = contract.clone();
        mutated
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("futureProperty".to_string(), serde_json::json!(true));
        assert!(
            ui_generator::validate_contract(&mutated).is_err(),
            "unknown property accepted at {pointer}"
        );
    }
}

#[test]
fn generated_ui_artifacts_project_every_contract_section() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let contract = ui_generator::load_contract(&root.join("resources/extension-ui")).unwrap();
    let rust = include_str!(concat!(env!("OUT_DIR"), "/extension_ui_contract.rs"));
    let typescript = ui_generator::render_typescript(&contract).unwrap();
    let node = ui_generator::render_node(&contract).unwrap();
    let sdk = ui_generator::render_sdk_contract(&contract).unwrap();

    let expected_keys = [
        "apiVersion",
        "modes",
        "contributionTypes",
        "primitives",
        "themeBases",
        "locales",
        "placementOperations",
        "placements",
        "protectedOccupants",
        "icons",
        "themeTokens",
        "limits",
        "validation",
        "loadingStages",
        "diagnosticCodes",
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    let actual_keys = contract
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(actual_keys, expected_keys);

    for expected in [
        "UI_MODES",
        "UI_CONTRIBUTION_TYPES",
        "UI_PRIMITIVES",
        "UI_THEME_BASES",
        "UI_LOCALES",
        "UI_PLACEMENT_OPERATIONS",
        "pub struct UiPlacement",
        "pub const UI_PLACEMENTS",
        "pub struct UiProtectedOccupant",
        "pub const UI_PROTECTED_OCCUPANTS",
        "pub struct UiValidation",
        "pub const UI_VALIDATION",
    ] {
        assert!(rust.contains(expected), "Rust omitted {expected}");
    }
    for field in [
        "contribution_type:",
        "cardinality:",
        "scope:",
        "third_party_chat_allowed:",
        "occupant:",
        "operations:",
        "max_numeric_limit:",
        "min_order:",
        "max_order:",
        "theme_value_pattern:",
    ] {
        assert!(rust.contains(field), "Rust omitted field {field}");
    }
    assert_eq!(
        rust.matches("UiPlacement { key:").count(),
        contract["placements"].as_array().unwrap().len()
    );
    assert_eq!(
        rust.matches("UiProtectedOccupant { placement:").count(),
        contract["protectedOccupants"].as_array().unwrap().len()
    );
    for artifact in [&typescript, &node, &sdk] {
        for key in &expected_keys {
            let projection = serde_json::to_string(&contract[*key]).unwrap();
            assert!(
                artifact.contains(&projection),
                "generated artifact omitted {key}"
            );
        }
    }
}

#[test]
fn sdk_requires_new_tools_to_declare_their_effect() {
    let sdk = include_str!("../../../resources/extension-host/sdk/index.d.ts");

    assert!(sdk.contains("effect: ExtensionEffectClass;"));
    assert!(!sdk.contains("effect?: ExtensionEffectClass;"));
}

#[test]
fn absent_private_document_is_allowed_for_a_clean_clone() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("docs/private.md");

    generator::update_private_document_if_present(&path, "generated").unwrap();

    assert!(!path.exists());
}

#[test]
fn fixed_bootstrap_anchors_match_the_node_reader() {
    let bootstrap: serde_json::Value = serde_json::from_str(include_str!(
        "../../../resources/extension-host/contract-bootstrap.json"
    ))
    .unwrap();
    let node_reader = include_str!("../../../resources/extension-host/contract.mjs");

    assert_eq!(bootstrap.as_object().unwrap().len(), 1);
    assert_eq!(bootstrap["maxContractBytes"], 8_192);
    assert!(
        include_bytes!("../../../resources/extension-host/contract-bootstrap.json").len()
            <= generator::BOOTSTRAP_FILE_MAX_BYTES
    );
    assert_eq!(
        node_numeric_constant(node_reader, "BOOTSTRAP_FILE_MAX_BYTES"),
        generator::BOOTSTRAP_FILE_MAX_BYTES
    );
    assert_eq!(
        node_numeric_constant(node_reader, "MAX_BOOTSTRAPPED_CONTRACT_BYTES"),
        generator::MAX_BOOTSTRAPPED_CONTRACT_BYTES
    );
    assert!(node_reader.matches("readBounded(").count() >= 2);
}

fn node_numeric_constant(source: &str, name: &str) -> usize {
    let prefix = format!("export const {name} = ");
    source
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .and_then(|value| value.strip_suffix(';'))
        .map(|value| value.replace('_', ""))
        .and_then(|value| value.parse().ok())
        .expect("generated Node bootstrap anchor")
}

#[test]
fn checked_in_sdk_readme_tables_match_the_contract() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let contract = generator::load_contract(&root.join("resources/extension-host")).unwrap();
    let expected = generator::generated_document_section(&contract).unwrap();
    // Git extrait les fichiers texte en CRLF sur certains runners Windows ;
    // le contrat porte le contenu, pas le séparateur de lignes de la copie.
    let sdk = include_str!("../../../resources/extension-host/sdk/README.md").replace("\r\n", "\n");

    assert!(sdk.contains(&expected));
}

#[test]
fn checked_in_private_document_tables_match_the_contract() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let contract = generator::load_contract(&root.join("resources/extension-host")).unwrap();
    let expected = generator::generated_document_section(&contract).unwrap();
    let path = generator::private_api_path(root).unwrap();
    if !path.exists() {
        // This documentation is private and intentionally absent from clean
        // clones; public generated artifacts remain mandatory above.
        return;
    }
    let private = std::fs::read_to_string(path).unwrap();

    assert!(private.contains(&expected));
}

#[test]
fn replaced_local_contract_constants_do_not_return() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for (path, forbidden) in [
        (
            "src/services/extensions/core_bridge.rs",
            "const MAX_SESSION_RESULTS",
        ),
        (
            "src/services/extensions/core_bridge.rs",
            "const MAX_PROJECT_RESULTS",
        ),
        (
            "src/services/extensions/core_bridge.rs",
            "const MCP_CALL_TIMEOUT",
        ),
        (
            "src/services/extensions/runtime_restart.rs",
            "const AUTO_RESTART_LIMIT",
        ),
        (
            "src/services/extensions/runtime_restart.rs",
            "const AUTO_RESTART_WINDOW",
        ),
        (
            "src/services/extensions/types.rs",
            "pub const BEAVER_API_VERSION",
        ),
        (
            "resources/extension-host/protocol.mjs",
            "const REQUEST_TIMEOUT_MS",
        ),
        (
            "resources/extension-host/loader.mjs",
            "const TOOL_TIMEOUT_MS",
        ),
        (
            "resources/extension-host/extension-api.mjs",
            "const EVENT_TIMEOUT_MS",
        ),
    ] {
        assert!(!std::fs::read_to_string(root.join(path))
            .unwrap()
            .contains(forbidden));
    }
}

#[test]
#[ignore = "developer command that refreshes checked-in extension contract artifacts"]
fn export_extension_contract_artifacts() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    generator::export_artifacts(root).unwrap();
}

#[test]
fn checked_in_ui_contract_artifacts_name_the_json_authority() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let contract = ui_generator::load_contract(&root.join("resources/extension-ui")).unwrap();
    let typescript =
        std::fs::read_to_string(root.join("../src/types/extension-ui-contract.generated.ts"))
            .unwrap();
    let sdk = std::fs::read_to_string(root.join("resources/extension-host/sdk/ui-contract.d.ts"))
        .unwrap();
    let node =
        std::fs::read_to_string(root.join("resources/extension-host/ui-contract.mjs")).unwrap();

    assert_eq!(
        typescript.replace("\r\n", "\n"),
        ui_generator::render_typescript(&contract).unwrap()
    );
    assert_eq!(
        sdk.replace("\r\n", "\n"),
        ui_generator::render_sdk_contract(&contract).unwrap()
    );
    assert_eq!(
        node.replace("\r\n", "\n"),
        ui_generator::render_node(&contract).unwrap()
    );
}

#[test]
fn sdk_readme_has_one_bounded_ui_generated_section() {
    let readme = include_str!("../../../resources/extension-host/sdk/README.md")
        .replace("\r\n", "\n");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let contract = ui_generator::load_contract(&root.join("resources/extension-ui")).unwrap();
    let expected = ui_generator::generated_document_section(&contract).unwrap();

    assert_eq!(readme.matches(UI_GENERATED_BEGIN).count(), 1);
    assert_eq!(readme.matches(UI_GENERATED_END).count(), 1);
    assert!(readme.find(UI_GENERATED_BEGIN).unwrap() < readme.find(UI_GENERATED_END).unwrap());
    assert!(readme.contains(&expected));
}

#[test]
#[ignore = "developer command that refreshes checked-in extension UI contract artifacts"]
fn export_extension_ui_contract_artifacts() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    ui_generator::export_artifacts(root).unwrap();
}
