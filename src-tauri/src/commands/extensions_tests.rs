use crate::services::extensions::{extension_recovery, loading_marker, registry_recovery};

#[test]
fn recovery_state_is_bounded_and_retry_stops_after_three_attempts() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    loading_marker::start_at(&path, "com.example.crash", 2).unwrap();

    let retryable = extension_recovery::state_at(&path, false);
    assert_eq!(retryable.extension_id.as_deref(), Some("com.example.crash"));
    assert_eq!(retryable.stage.as_deref(), Some("import"));
    assert_eq!(retryable.attempts, Some(2));
    assert!(retryable.can_retry);
    assert!(!retryable.marker_invalid);

    loading_marker::start_at(&path, "com.example.crash", 3).unwrap();
    assert!(!extension_recovery::state_at(&path, false).can_retry);

    std::fs::write(&path, b"{secret-url-and-path-sentinel}").unwrap();
    let invalid = extension_recovery::state_at(&path, true);
    assert!(invalid.marker_invalid);
    assert!(invalid.extension_id.is_none());
    assert!(invalid.stage.is_none());
    assert!(invalid.attempts.is_none());
    assert!(invalid.recovery_snapshot_available);
    let visible = serde_json::to_string(&invalid).unwrap();
    assert!(!visible.contains("secret-url-and-path-sentinel"));
    assert!(!visible.contains(directory.path().to_string_lossy().as_ref()));
}

#[test]
fn an_orphaned_but_well_formed_marker_is_only_discardable() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    loading_marker::start_at(&path, "com.example.removed", 1).unwrap();

    let state = extension_recovery::state_at_with_records(&path, false, &[]);
    assert!(state.marker_invalid);
    assert!(state.extension_id.is_none());
    extension_recovery::discard_marker_at(&path, &[]).unwrap();
    assert!(matches!(
        loading_marker::read_at(&path),
        loading_marker::MarkerRead::Missing
    ));
}

#[test]
fn global_recovery_cancel_restores_only_installed_and_still_trusted_ids() {
    let mut enabled = fixture("com.example.enabled", true, true);
    let mut revoked = fixture("com.example.revoked", true, false);
    let mut disabled = fixture("com.example.disabled", false, true);
    let mut records = vec![enabled.clone(), revoked.clone(), disabled.clone()];

    let snapshot = registry_recovery::disable_records_and_snapshot(&mut records);
    assert_eq!(snapshot, vec!["com.example.enabled", "com.example.revoked"]);
    assert!(records.iter().all(|record| !record.enabled));

    assert!(registry_recovery::restore_records(
        &mut records,
        Some(snapshot)
    ));
    enabled.enabled = true;
    revoked.enabled = false;
    disabled.enabled = false;
    assert_eq!(records, vec![enabled, revoked, disabled]);
    assert!(!registry_recovery::restore_records(
        &mut records,
        Some(Vec::new())
    ));
    assert!(!registry_recovery::restore_records(&mut records, None));
}

#[test]
fn repeated_global_recovery_preserves_the_first_durable_snapshot() {
    let mut records = vec![fixture("com.example.enabled", true, true)];
    let mut snapshot = None;

    registry_recovery::disable_records_preserving_snapshot(&mut records, &mut snapshot);
    assert_eq!(snapshot, Some(vec!["com.example.enabled".to_string()]));
    registry_recovery::disable_records_preserving_snapshot(&mut records, &mut snapshot);
    assert_eq!(snapshot, Some(vec!["com.example.enabled".to_string()]));
}

fn fixture(
    id: &str,
    enabled: bool,
    trusted: bool,
) -> crate::services::extensions::types::ExtensionRecord {
    use crate::services::extensions::types::*;
    ExtensionRecord {
        manifest: ExtensionManifest {
            id: id.to_string(),
            name: id.to_string(),
            version: "1.0.0".to_string(),
            beaver_api: "1".to_string(),
            runtime: "node".to_string(),
            main: Some("index.mjs".to_string()),
            ui: None,
            access: "full".to_string(),
            api_level: ExtensionApiLevel::Stable,
            essential: false,
            author: None,
            homepage: None,
            description: None,
        },
        kind: ExtensionKind::Local,
        source: "test".to_string(),
        origin: None,
        enabled,
        trusted,
        fingerprint: None,
        trusted_at: None,
        show_in_chat: true,
        status: ExtensionStatus::Inactive,
        last_error: None,
        last_activated_at: None,
        sensitive_access_granted: false,
        contributions: ExtensionContributions::default(),
    }
}
