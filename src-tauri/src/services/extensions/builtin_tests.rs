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
    assert!(super::types::MAX_EXTENSIONS >= records.len() + super::types::MAX_USER_EXTENSIONS);
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
    let mut external = records[0].clone();
    external.kind = super::types::ExtensionKind::External;
    external.manifest.id = "com.example.external".to_string();
    records.extend([local, external]);

    super::registry::disable_hosted_records(&mut records);

    assert!(records
        .iter()
        .filter(|record| record.kind != super::types::ExtensionKind::External)
        .all(|record| !record.enabled));
    assert!(
        records
            .iter()
            .find(|record| record.kind == super::types::ExtensionKind::External)
            .unwrap()
            .enabled
    );
}
