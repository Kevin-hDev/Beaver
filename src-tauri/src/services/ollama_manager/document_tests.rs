use super::fingerprint::{BundleFingerprint, OllamaVersion, Sha256Digest};
use super::journal::{
    classify_migration_marker, OllamaJournalState, OllamaMigrationMarker,
    OllamaMigrationMarkerClassification, OllamaTransactionJournal,
};
use crate::services::paths::{ollama_paths, OllamaPaths};
use serde_json::{json, Value};
use std::path::Path;

fn digest(hex: &str) -> Sha256Digest {
    Sha256Digest::from_hex(hex).expect("valid digest fixture")
}

fn fingerprint(version: &str, hex: &str) -> BundleFingerprint {
    BundleFingerprint {
        version: OllamaVersion::parse(version).expect("valid version fixture"),
        executable_sha256: digest(hex),
    }
}

fn journal(state: OllamaJournalState) -> OllamaTransactionJournal {
    OllamaTransactionJournal::new(state)
}

#[test]
fn journal_wire_stays_private_to_bounded_journal_module() {
    let facade = include_str!("../ollama_manager.rs");
    let journal_module = include_str!("journal.rs");

    assert!(
        journal_module.contains("mod journal_wire;"),
        "wire parser must be nested under journal.rs"
    );
    assert!(
        !facade.contains("mod journal_wire;"),
        "wire parser must not be a sibling visible from the manager facade"
    );
}

fn assert_direct_children(paths: &OllamaPaths, root: &Path) {
    let values = [
        &paths.active,
        &paths.legacy_staging,
        &paths.legacy_backup,
        &paths.failed,
        &paths.install_staging,
        &paths.archive_staging,
        &paths.archive_failed,
        &paths.uncommitted_staging_delete,
        &paths.update_staging,
        &paths.backup,
        &paths.backup_delete,
        &paths.failed_delete,
        &paths.journal,
        &paths.journal_tmp,
        &paths.migration_marker,
        &paths.migration_marker_tmp,
        &paths.process_receipt,
        &paths.probe_models,
    ];
    for path in values {
        assert_eq!(path.parent(), Some(root));
    }
    for (index, left) in values.iter().enumerate() {
        for right in values.iter().skip(index + 1) {
            assert_ne!(left, right);
        }
    }
}

#[test]
fn ollama_paths_keep_all_named_children_under_the_received_root() {
    let root = Path::new("/tmp/beaver-task-3-data");
    let paths = ollama_paths(root);

    assert_direct_children(&paths, root);
    assert_eq!(
        paths.active.file_name().and_then(|name| name.to_str()),
        Some("ollama-bundle")
    );
    assert_eq!(
        paths
            .legacy_staging
            .file_name()
            .and_then(|name| name.to_str()),
        Some("ollama-bundle-staging")
    );
    assert_eq!(
        paths
            .legacy_backup
            .file_name()
            .and_then(|name| name.to_str()),
        Some("ollama-bundle-old")
    );
    assert_eq!(
        paths.failed.file_name().and_then(|name| name.to_str()),
        Some("ollama-bundle-failed")
    );
    assert_eq!(
        paths
            .install_staging
            .file_name()
            .and_then(|name| name.to_str()),
        Some("ollama-bundle-install-staging")
    );
    assert_eq!(
        paths
            .uncommitted_staging_delete
            .file_name()
            .and_then(|name| name.to_str()),
        Some("ollama-bundle-uncommitted-staging-delete")
    );
    assert_eq!(
        paths
            .update_staging
            .file_name()
            .and_then(|name| name.to_str()),
        Some("ollama-bundle-update-staging")
    );
    assert_eq!(
        paths.backup.file_name().and_then(|name| name.to_str()),
        Some("ollama-bundle-backup")
    );
    assert_eq!(
        paths
            .backup_delete
            .file_name()
            .and_then(|name| name.to_str()),
        Some("ollama-bundle-backup-delete")
    );
    assert_eq!(
        paths
            .failed_delete
            .file_name()
            .and_then(|name| name.to_str()),
        Some("ollama-bundle-failed-delete")
    );
    assert_eq!(
        paths.journal.file_name().and_then(|name| name.to_str()),
        Some("ollama-update-state.json")
    );
    assert_eq!(
        paths.journal_tmp.file_name().and_then(|name| name.to_str()),
        Some("ollama-update-state.tmp")
    );
    assert_eq!(
        paths
            .migration_marker
            .file_name()
            .and_then(|name| name.to_str()),
        Some("ollama-layout-migration.json")
    );
    assert_eq!(
        paths
            .migration_marker_tmp
            .file_name()
            .and_then(|name| name.to_str()),
        Some("ollama-layout-migration.tmp")
    );
    assert_eq!(
        paths
            .process_receipt_tmp
            .file_name()
            .and_then(|name| name.to_str()),
        Some("ollama-process-receipt.tmp")
    );
}

#[test]
fn version_accepts_only_canonical_semver_with_an_octet_bound() {
    for raw in ["0.1.0", "1.2.3-rc.1", "1.2.3+build.7"] {
        let parsed = OllamaVersion::parse(raw).expect("canonical semver");
        assert_eq!(parsed.as_str(), raw);
        assert_eq!(
            serde_json::to_string(&parsed).unwrap(),
            format!("\"{raw}\"")
        );
    }
    for raw in ["", "v1.2.3", " 1.2.3", "1.2.3 ", "1.2", "1.2.3.4", "1.02.3"] {
        assert!(
            OllamaVersion::parse(raw).is_err(),
            "accepted non-canonical {raw:?}"
        );
    }
    let long = format!("1.2.3+{}", "a".repeat(59));
    assert!(long.len() > 64);
    assert!(OllamaVersion::parse(&long).is_err());
    let utf8 = format!("1.2.3+{}", "é".repeat(30));
    assert!(utf8.len() > 64);
    assert!(OllamaVersion::parse(&utf8).is_err());
}

#[test]
fn sha_accepts_hex_case_but_serializes_lowercase_and_compares_constantly() {
    let lower = digest(&"ab".repeat(32));
    let upper = digest(&"AB".repeat(32));
    assert_eq!(lower.to_hex(), "ab".repeat(32));
    assert!(lower.constant_time_eq(&upper));
    assert_eq!(
        serde_json::to_string(&upper).unwrap(),
        format!("\"{}\"", "ab".repeat(32))
    );
    for raw in ["", "ab", &"a".repeat(63), &"a".repeat(65), &"gg".repeat(32)] {
        assert!(
            Sha256Digest::from_hex(raw).is_err(),
            "accepted invalid SHA {raw:?}"
        );
    }
    let non_ascii = format!("{}é", "a".repeat(62));
    assert!(Sha256Digest::from_hex(&non_ascii).is_err());
}

#[test]
fn each_journal_phase_round_trips_only_its_legal_fields() {
    let target = fingerprint("1.2.3", &"11".repeat(32));
    let previous = fingerprint("1.2.2", &"22".repeat(32));
    let rejected = fingerprint("1.2.4", &"33".repeat(32));
    let states = [
        OllamaJournalState::Prepared {
            target: target.clone(),
            previous: previous.clone(),
        },
        OllamaJournalState::PendingValidation {
            target: target.clone(),
            previous: previous.clone(),
        },
        OllamaJournalState::CleanupPending {
            target: target.clone(),
            previous: previous.clone(),
        },
        OllamaJournalState::RollbackPending {
            previous: previous.clone(),
            rejected_target: Some(rejected.clone()),
        },
        OllamaJournalState::RollbackCleanupPending {
            previous: previous.clone(),
            rejected_target: None,
        },
    ];
    for state in states {
        let encoded = serde_json::to_vec(&journal(state)).expect("serialize journal");
        let decoded = OllamaTransactionJournal::parse_bounded(&encoded).expect("parse journal");
        assert_eq!(
            decoded,
            OllamaTransactionJournal::parse_bounded(&encoded).unwrap()
        );
    }
}

#[test]
fn duplicate_root_and_nested_fields_fail_closed() {
    let previous = format!(
        "{{\"version\":\"1.2.2\",\"executable_sha256\":\"{}\"}}",
        "22".repeat(32)
    );
    let target = format!(
        "{{\"version\":\"1.2.3\",\"executable_sha256\":\"{}\"}}",
        "11".repeat(32)
    );
    let fixtures = [
        format!(
            "{{\"schema_version\":2,\"schema_version\":1,\"phase\":\"Prepared\",\"target\":{},\"previous\":{}}}",
            target, previous
        ),
        format!(
            "{{\"schema_version\":1,\"phase\":\"RollbackPending\",\"phase\":\"Prepared\",\"target\":{},\"previous\":{}}}",
            target, previous
        ),
        format!(
            "{{\"schema_version\":1,\"phase\":\"Prepared\",\"target\":{{\"version\":\"1.2.3\",\"version\":\"1.2.3\",\"executable_sha256\":\"{}\"}},\"previous\":{}}}",
            "11".repeat(32), previous
        ),
        format!(
            "{{\"schema_version\":1,\"phase\":\"Prepared\",\"target\":{{\"version\":\"1.2.3\",\"executable_sha256\":\"{}\",\"executable_sha256\":\"{}\"}},\"previous\":{}}}",
            "11".repeat(32), "11".repeat(32), previous
        ),
        format!(
            "{{\"schema_version\":1,\"phase\":\"Prepared\",\"target\":{},\"previous\":{},\"previous\":{}}}",
            target, previous, previous
        ),
        format!(
            "{{\"schema_version\":1,\"phase\":\"RollbackPending\",\"previous\":{},\"rejected_target\":null,\"rejected_target\":null}}",
            previous
        ),
    ];
    for fixture in fixtures {
        assert!(
            OllamaTransactionJournal::parse_bounded(fixture.as_bytes()).is_err(),
            "duplicate JSON key was accepted: {fixture}"
        );
    }
}

#[test]
fn journal_uses_exact_phase_fields_and_rejects_illegal_shapes() {
    let target = json!(fingerprint("1.2.3", &"11".repeat(32)));
    let previous = json!(fingerprint("1.2.2", &"22".repeat(32)));
    for phase in ["Prepared", "PendingValidation", "CleanupPending"] {
        let value =
            json!({"schema_version": 1, "phase": phase, "target": target, "previous": previous});
        let object = serde_json::from_value::<Value>(value).unwrap();
        assert!(
            OllamaTransactionJournal::parse_bounded(&serde_json::to_vec(&object).unwrap()).is_ok()
        );
    }
    for phase in ["RollbackPending", "RollbackCleanupPending"] {
        let value = json!({"schema_version": 1, "phase": phase, "previous": previous, "rejected_target": null});
        assert!(
            OllamaTransactionJournal::parse_bounded(&serde_json::to_vec(&value).unwrap()).is_ok()
        );
    }
    let targetless = json!({"schema_version": 1, "phase": "RollbackPending", "target": target, "previous": previous, "rejected_target": null});
    assert!(
        OllamaTransactionJournal::parse_bounded(&serde_json::to_vec(&targetless).unwrap()).is_err()
    );
    let rejected_on_prepared = json!({"schema_version": 1, "phase": "Prepared", "target": target, "previous": previous, "rejected_target": null});
    assert!(OllamaTransactionJournal::parse_bounded(
        &serde_json::to_vec(&rejected_on_prepared).unwrap()
    )
    .is_err());
    let source =
        json!({"schema_version": 1, "phase": "Prepared", "source": target, "previous": previous});
    assert!(
        OllamaTransactionJournal::parse_bounded(&serde_json::to_vec(&source).unwrap()).is_err()
    );
    let missing_previous = json!({"schema_version": 1, "phase": "Prepared", "target": target});
    assert!(OllamaTransactionJournal::parse_bounded(
        &serde_json::to_vec(&missing_previous).unwrap()
    )
    .is_err());
    let unknown_phase = json!({"schema_version": 1, "phase": "Unknown", "previous": previous});
    assert!(
        OllamaTransactionJournal::parse_bounded(&serde_json::to_vec(&unknown_phase).unwrap())
            .is_err()
    );
    let unknown_schema =
        json!({"schema_version": 2, "phase": "Prepared", "target": target, "previous": previous});
    assert!(
        OllamaTransactionJournal::parse_bounded(&serde_json::to_vec(&unknown_schema).unwrap())
            .is_err()
    );
    let unknown_field = json!({"schema_version": 1, "phase": "Prepared", "target": target, "previous": previous, "extra": true});
    assert!(
        OllamaTransactionJournal::parse_bounded(&serde_json::to_vec(&unknown_field).unwrap())
            .is_err()
    );
}

#[test]
fn oversized_documents_are_rejected_before_deserialization() {
    let oversized = vec![b' '; 4097];
    assert!(OllamaTransactionJournal::parse_bounded(&oversized).is_err());
    assert!(matches!(
        classify_migration_marker(Some(&oversized)),
        OllamaMigrationMarkerClassification::Invalid
    ));
}

#[test]
fn valid_json_padded_to_4097_bytes_is_rejected_by_the_bounded_reader() {
    let mut padded = serde_json::to_vec(&journal(OllamaJournalState::Prepared {
        target: fingerprint("1.2.3", &"11".repeat(32)),
        previous: fingerprint("1.2.2", &"22".repeat(32)),
    }))
    .unwrap();
    assert!(padded.len() < 4097);
    padded.resize(4097, b' ');
    assert!(serde_json::from_slice::<Value>(&padded).is_ok());
    assert!(OllamaTransactionJournal::parse_bounded(&padded).is_err());
}

#[test]
fn migration_marker_distinguishes_absent_valid_and_invalid() {
    assert!(matches!(
        classify_migration_marker(None),
        OllamaMigrationMarkerClassification::Absent
    ));
    let valid = serde_json::to_vec(&OllamaMigrationMarker::new()).unwrap();
    assert!(matches!(
        classify_migration_marker(Some(&valid)),
        OllamaMigrationMarkerClassification::Valid(_)
    ));
    for value in [
        json!({"schema_version": 2, "legacy_layout_migrated": true}),
        json!({"schema_version": 1, "legacy_layout_migrated": false}),
        json!({"schema_version": 1, "legacy_layout_migrated": true, "extra": 1}),
        json!({"schema_version": 1}),
    ] {
        let bytes = serde_json::to_vec(&value).unwrap();
        assert!(matches!(
            classify_migration_marker(Some(&bytes)),
            OllamaMigrationMarkerClassification::Invalid
        ));
    }
    for fixture in [
        br#"{"schema_version":2,"schema_version":1,"legacy_layout_migrated":true}"#.as_slice(),
        br#"{"schema_version":1,"legacy_layout_migrated":true,"legacy_layout_migrated":false}"#
            .as_slice(),
    ] {
        assert!(matches!(
            classify_migration_marker(Some(fixture)),
            OllamaMigrationMarkerClassification::Invalid
        ));
    }
}
