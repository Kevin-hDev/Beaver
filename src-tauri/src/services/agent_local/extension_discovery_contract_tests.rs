#[path = "../../../extension_contract_shared.rs"]
#[allow(dead_code)]
mod extension_contract_shared;
#[path = "../../../extension_discovery_contract_build.rs"]
#[allow(dead_code)]
mod generator;

#[test]
fn discovery_contract_defines_the_r0_names_limits_and_host_imports() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let discovery = generator::load_contract(&root.join("resources/extension-discovery")).unwrap();
    let host = host_contract(root);

    generator::validate_contract(&discovery, &host).unwrap();
    assert_eq!(
        discovery["toolNames"],
        serde_json::json!([
            "list_extensions",
            "inspect_extensions",
            "load_extension_resource"
        ])
    );
    assert_eq!(discovery["limits"]["contextThresholdPercent"], 10);
    assert_eq!(discovery["limits"]["unknownContextTokens"], 20_000);
    assert_eq!(discovery["limits"]["maxInspectedExtensions"], 4);
    assert_eq!(discovery["limits"]["maxCompactCatalogBytes"], 32_768);
    assert_eq!(discovery["limits"]["maxSerializedResultBytes"], 393_216);
    assert_eq!(
        discovery["imports"],
        serde_json::json!([
            "maxExtensions",
            "maxToolsPerExtension",
            "maxSkillsPerExtension",
            "maxResourcesPerExtension",
            "maxIdentifierChars"
        ])
    );
}

#[test]
fn discovery_contract_rejects_duplicate_or_copied_authority_keys() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let discovery = generator::load_contract(&root.join("resources/extension-discovery")).unwrap();
    let host = host_contract(root);

    let mut duplicate_name = discovery.clone();
    duplicate_name["toolNames"][1] = duplicate_name["toolNames"][0].clone();
    assert!(generator::validate_contract(&duplicate_name, &host).is_err());

    let mut copied_limit = discovery.clone();
    copied_limit["limits"]["maxExtensions"] = host["limits"]["maxExtensions"].clone();
    assert!(generator::validate_contract(&copied_limit, &host).is_err());

    let mut hostile_import = host.clone();
    hostile_import["limits"]["maxExtensions"] = serde_json::json!(1_073_741_824_u64);
    assert!(generator::validate_contract(&discovery, &hostile_import).is_err());

    let mut hostile_identifier = host.clone();
    hostile_identifier["limits"]["maxIdentifierChars"] = serde_json::json!(1_073_741_824_u64);
    assert!(generator::validate_contract(&discovery, &hostile_identifier).is_err());

    let mut too_small_identifier = host;
    too_small_identifier["limits"]["maxIdentifierChars"] = serde_json::json!(3);
    assert!(generator::validate_contract(&discovery, &too_small_identifier).is_err());
}

#[test]
fn discovery_bootstrap_and_generated_rust_are_bounded() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("contract-bootstrap.json"),
        vec![b' '; extension_contract_shared::BOOTSTRAP_FILE_MAX_BYTES + 1],
    )
    .unwrap();
    std::fs::write(directory.path().join("contract.json"), b"{}").unwrap();
    assert!(generator::load_contract(directory.path()).is_err());

    let generated = include_str!(concat!(env!("OUT_DIR"), "/extension_discovery_contract.rs"));
    assert!(generated.contains("pub const CONTEXT_THRESHOLD_PERCENT: usize = 10;"));
    assert!(generated.contains("pub const HOST_MAX_EXTENSIONS: usize = 132;"));
    assert!(generated.contains("pub const HOST_MAX_TOOLS_PER_EXTENSION: usize = 64;"));
    assert!(generated.contains("pub const HOST_MAX_IDENTIFIER_CHARS: usize = 96;"));
    assert!(generated.contains("pub const DISCOVERY_TOOL_NAMES"));
}

#[test]
fn validation_proves_worst_case_json_budgets_without_truncation() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let discovery = generator::load_contract(&root.join("resources/extension-discovery")).unwrap();
    assert!(generator::validate_contract(&discovery, &host_contract(root)).is_ok());

    let mut compact_too_small = discovery.clone();
    compact_too_small["limits"]["maxCompactCatalogBytes"] = serde_json::json!(1);
    assert!(generator::validate_contract(&compact_too_small, &host_contract(root)).is_err());

    let mut result_too_small = discovery;
    result_too_small["limits"]["maxSerializedResultBytes"] = serde_json::json!(1);
    assert!(generator::validate_contract(&result_too_small, &host_contract(root)).is_err());
}

fn host_contract(root: &std::path::Path) -> serde_json::Value {
    serde_json::from_slice(
        &std::fs::read(root.join("resources/extension-host/contract.json")).unwrap(),
    )
    .unwrap()
}
