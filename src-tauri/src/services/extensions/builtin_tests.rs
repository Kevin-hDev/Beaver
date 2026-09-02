#[test]
fn catalog_contains_only_the_office_suite() {
    let records = super::builtin::records().unwrap();
    let ids = records
        .iter()
        .map(|record| record.manifest.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ids,
        [
            "beaver.office.documents",
            "beaver.office.pdf",
            "beaver.office.spreadsheets",
            "beaver.office.presentations",
        ]
    );
    assert!(records.iter().all(|record| {
        record.kind == super::types::ExtensionKind::Builtin
            && record.enabled
            && record.trusted
            && record.manifest.runtime == "node"
    }));
    assert_eq!(
        super::types::MAX_EXTENSIONS,
        records.len() + super::types::MAX_USER_EXTENSIONS
    );
}

#[test]
fn preferences_survive_catalog_updates() {
    let mut stored = super::builtin::records().unwrap();
    stored[0].enabled = false;
    stored[0].show_in_chat = false;
    stored[0].manifest.version = "0.1.0".to_string();

    let merged = super::builtin::merge(stored).unwrap();

    assert!(!merged[0].enabled);
    assert!(!merged[0].show_in_chat);
    assert_eq!(merged[0].manifest.version, "1.0.0");
}

#[test]
fn recovery_disables_builtin_and_local_extensions() {
    let mut records = super::builtin::records().unwrap();
    let mut local = records[0].clone();
    local.kind = super::types::ExtensionKind::Local;
    local.manifest.id = "com.example.local".to_string();
    records.push(local);

    super::registry_recovery::disable_hosted_records(&mut records);

    assert!(records.iter().all(|record| !record.enabled));
}

#[test]
fn every_builtin_documents_its_model_experience_effect() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/extension-host");
    let catalog: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("builtin-plugins/catalog.json")).unwrap(),
    )
    .unwrap();

    for plugin in catalog["plugins"].as_array().unwrap() {
        let entry = plugin["manifest"]["main"].as_str().unwrap();
        let readme =
            std::fs::read_to_string(root.join(entry).parent().unwrap().join("README.md")).unwrap();
        let documented = readme.contains("## Effet sur le modèle")
            && readme.contains("### Coût")
            && readme.contains("### Cache")
            && readme.contains("### Surface visible");
        assert!(documented || readme.contains("NO_MODEL_EXPERIENCE_SECTION"));
    }
}

#[test]
fn discovery_terms_are_absent_from_contract_manifests_and_sdk() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/extension-host");
    for path in [
        root.join("contract.json"),
        root.join("builtin-plugins/catalog.json"),
        root.join("sdk/index.d.ts"),
        root.join("sdk/contract.d.ts"),
    ] {
        let source = std::fs::read_to_string(path).unwrap();
        assert!(!source.contains("discoveryTerms"));
    }
}
