use super::types::{
    ExtensionApiLevel, ExtensionManifest, ExtensionTool, BEAVER_API_VERSION,
    MAX_TOOLS_PER_EXTENSION,
};
use serde_json::json;

fn manifest(id: &str) -> ExtensionManifest {
    ExtensionManifest {
        id: id.to_string(),
        name: "Test Extension".to_string(),
        version: "1.0.0".to_string(),
        beaver_api: BEAVER_API_VERSION.to_string(),
        runtime: "node".to_string(),
        main: Some("index.ts".to_string()),
        ui: None,
        access: "full".to_string(),
        api_level: ExtensionApiLevel::Stable,
        essential: false,
        author: None,
        homepage: None,
        description: None,
    }
}

#[test]
fn manifest_rejects_traversal_and_invalid_identifiers() {
    let mut traversal = manifest("com.example.safe");
    traversal.main = Some("../outside.ts".to_string());
    assert!(super::validation::manifest(&traversal).is_err());

    let invalid = manifest("bad id");
    assert!(super::validation::manifest(&invalid).is_err());

    let trailing_separator = manifest("com.example.");
    assert!(super::validation::manifest(&trailing_separator).is_err());
}

#[test]
fn manifest_rejects_incompatible_api() {
    let mut incompatible = manifest("com.example.incompatible");
    incompatible.beaver_api = "99".to_string();

    assert!(super::validation::manifest(&incompatible).is_err());
}

#[test]
fn node_extensions_cannot_claim_restricted_access() {
    let mut misleading = manifest("com.example.misleading");
    misleading.access = "core".to_string();

    assert!(super::validation::manifest(&misleading).is_err());
}

#[test]
fn contributions_are_bounded_and_require_object_schemas() {
    let invalid = ExtensionTool {
        name: "com.example.invalid".to_string(),
        description: "Invalid schema".to_string(),
        parameters: json!({"type": "string"}),
        effect: super::types::ExtensionEffect::Unknown,
        replaces_core: false,
    };
    assert!(super::validation::contributions(&[invalid], &[]).is_err());

    let tools = (0..=MAX_TOOLS_PER_EXTENSION)
        .map(|index| ExtensionTool {
            name: format!("com.example.tool{index}"),
            description: "Tool".to_string(),
            parameters: json!({"type": "object"}),
            effect: super::types::ExtensionEffect::Unknown,
            replaces_core: false,
        })
        .collect::<Vec<_>>();
    assert!(super::validation::contributions(&tools, &[]).is_err());
}

#[test]
fn protocol_collections_are_explicitly_bounded() {
    let oversized = json!((0..4_097).collect::<Vec<_>>());

    assert!(super::validation::message(&oversized).is_err());
}

#[test]
fn storage_round_trip_is_bounded() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extensions.json");
    let records = Vec::new();

    super::storage::save_to(&path, &records, &None).unwrap();
    let loaded = super::storage::load_from(&path).unwrap();

    assert_eq!(loaded.extensions, records);
}

#[test]
fn runtime_contributions_are_not_persisted() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("extension.ts");
    let storage = directory.path().join("extensions.json");
    std::fs::write(&source, "export default function () {}").unwrap();
    let mut record = super::manifest::load_local(source.to_str().unwrap())
        .unwrap()
        .record;
    record.contributions.tools.push(ExtensionTool {
        name: "com.example.runtime".to_string(),
        description: "Runtime only".to_string(),
        parameters: json!({"type": "object"}),
        effect: super::types::ExtensionEffect::Unknown,
        replaces_core: false,
    });

    super::storage::save_to(&storage, &[record], &None).unwrap();
    let loaded = super::storage::load_from(&storage).unwrap();

    assert!(loaded.extensions[0].contributions.tools.is_empty());
}

#[test]
fn direct_typescript_react_entry_is_supported_by_jiti() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("extension.tsx");
    std::fs::write(&source, "export default function () {}").unwrap();

    let loaded = super::manifest::load_local(source.to_str().unwrap()).unwrap();

    assert_eq!(
        loaded.record.manifest.main.as_deref(),
        Some("extension.tsx")
    );
}

#[test]
fn stored_records_reject_duplicate_ids() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("extension.ts");
    std::fs::write(&source, "export default function () {}").unwrap();
    let record = super::manifest::load_local(source.to_str().unwrap())
        .unwrap()
        .record;

    assert!(super::validation::records(&[record.clone(), record]).is_err());
}

#[cfg(unix)]
#[test]
fn directory_manifest_cannot_escape_through_a_symlink() {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir().unwrap();
    let extension = directory.path().join("extension");
    let outside = directory.path().join("outside.json");
    std::fs::create_dir(&extension).unwrap();
    std::fs::write(&outside, "{}").unwrap();
    symlink(&outside, extension.join("beaver.json")).unwrap();

    assert!(super::manifest::load_local(extension.to_str().unwrap()).is_err());
}

#[tokio::test]
async fn unknown_core_calls_fail_closed() {
    let context = super::call_context::ExtensionCallContext::for_test(
        super::host_identity::HostIdentity::Official,
        super::types::ExtensionApiLevel::Stable,
    );
    let result = super::core_bridge::call(&context, "unknown.method", None).await;

    assert!(result.is_err());
}

#[test]
fn host_protocol_requires_a_json_rpc_envelope() {
    assert!(super::protocol::envelope(&json!({
        "jsonrpc": "2.0",
        "id": "request",
        "result": {}
    }))
    .is_ok());
    assert!(super::protocol::envelope(&json!({
        "jsonrpc": "1.0",
        "id": "request",
        "result": {}
    }))
    .is_err());
    assert!(super::protocol::envelope(&json!({
        "jsonrpc": "2.0",
        "result": {}
    }))
    .is_err());
    assert!(super::protocol::envelope(&json!({
        "jsonrpc": "2.0",
        "id": "x".repeat(129),
        "result": {}
    }))
    .is_err());
}
