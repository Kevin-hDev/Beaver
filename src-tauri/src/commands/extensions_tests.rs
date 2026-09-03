use crate::services::extensions::{extension_recovery, loading_marker, registry_recovery};

#[test]
fn every_extension_command_closes_unknown_errors() {
    let generated = crate::services::extensions::error_codes::ALL;

    assert_eq!(super::command_error::ExtensionCommand::ALL.len(), 22);
    for command in super::command_error::ExtensionCommand::ALL {
        let error = super::command_error::close(
            command,
            Err::<(), _>("secret sentinel at /Users/private https://internal\nstack".to_string()),
        )
        .unwrap_err();
        assert!(generated.contains(&error.as_str()), "{command:?}: {error}");
        assert_eq!(error, "extensions_operation_failed");
        assert!(!error.contains("sentinel"));
        assert!(!error.contains("/Users/"));
        assert!(!error.contains("https://"));
        assert!(!error.contains("stack"));
    }
}

#[test]
fn r0_boundary_codes_remain_declared_and_preserved() {
    let codes = [
        "extensions_operation_failed",
        "extensions_fingerprint_changed",
        "extensions_fingerprint_failed",
        "extensions_stop_unconfirmed",
        "extensions_registry_entry_ignored",
        "extensions_registry_migration_failed",
        "extensions_recovery_marker_invalid",
        "extensions_load_interrupted",
        "extensions_activation_confirmation_required",
        "extensions_not_found",
        "extensions_host_incompatible",
    ];
    for code in codes {
        assert!(crate::services::extensions::error_codes::ALL.contains(&code));
        assert_eq!(
            super::command_error::close(
                super::command_error::ExtensionCommand::List,
                Err::<(), _>(code.to_string()),
            ),
            Err(code.to_string())
        );
    }
}

#[test]
fn extension_command_inventory_names_all_twenty_two_boundaries() {
    let actual = super::command_error::ExtensionCommand::ALL.map(|command| command.label());
    assert_eq!(
        actual,
        [
            "list_extensions",
            "add_local_extension",
            "install_git_extension",
            "install_npm_extension",
            "update_extension",
            "remove_extension",
            "set_extension_enabled",
            "set_extension_show_in_chat",
            "reload_extension_host",
            "get_extension_host_status",
            "get_extension_ui_catalog",
            "invoke_extension_ui_action",
            "report_extension_ui_mount_failure",
            "get_extension_discovery_preferences",
            "set_extension_discovery_preferences",
            "recover_extension_host",
            "open_extension_source",
            "get_extension_recovery_state",
            "keep_extension_disabled",
            "retry_extension_load",
            "discard_extension_loading_marker",
            "restore_extension_recovery_snapshot",
        ]
    );
}

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
fn ui_startup_projection_is_bounded_and_contains_no_internal_data() {
    let state = crate::services::extensions::UiStartupState::resolved(
        crate::services::extensions::UiStartupMode::PendingInterruptedUi {
            extension_id: "com.example.ui".to_string(),
            stage: "mount".to_string(),
            attempts: 2,
        },
    );
    let projection = crate::commands::extensions_ui_startup::project(&state);
    let json = serde_json::to_value(projection).unwrap();

    assert_eq!(json["mode"]["kind"], "pendingInterruptedUi");
    assert_eq!(json["mode"]["extensionId"], "com.example.ui");
    assert_eq!(json["canRetry"], true);
    assert_eq!(json["thirdPartyLoadingAllowed"], false);
    let visible = json.to_string();
    assert!(!visible.contains("token"));
    assert!(!visible.contains("/Users/"));
    assert!(!visible.contains("extension-loading.json"));
}

#[test]
fn frontend_fallback_cannot_elevate_a_normal_backend_state() {
    let state = crate::services::extensions::UiStartupState::resolved(
        crate::services::extensions::UiStartupMode::Normal,
    );

    assert!(crate::commands::extensions_ui_startup::continue_safe(&state).is_err());
    assert_eq!(
        state.mode(),
        crate::services::extensions::UiStartupMode::Normal
    );
    assert!(state.third_party_loading_allowed());
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
            ui_legacy: None,
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
        ui_artifact: None,
        trusted_at: None,
        show_in_chat: true,
        status: ExtensionStatus::Inactive,
        last_error: None,
        last_activated_at: None,
        sensitive_access_granted: false,
        contributions: ExtensionContributions::default(),
    }
}
