use super::protocol::HostExtensionSpec;
use super::runtime_sync::accepts_contributions;
use super::types::{
    ExtensionApiLevel, ExtensionContributions, ExtensionEffect, ExtensionManifest, ExtensionTool,
};
use serde_json::json;

fn record(id: &str, kind: super::types::ExtensionKind) -> super::types::ExtensionRecord {
    super::types::ExtensionRecord {
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
        kind,
        source: "test".to_string(),
        origin: None,
        enabled: true,
        trusted: true,
        fingerprint: None,
        ui_artifact: None,
        trusted_at: None,
        show_in_chat: true,
        status: super::types::ExtensionStatus::Inactive,
        last_error: None,
        last_activated_at: None,
        sensitive_access_granted: false,
        contributions: ExtensionContributions::default(),
    }
}

#[test]
fn typed_legacy_ui_stays_inactive_and_emits_the_runtime_diagnostic() {
    let mut legacy = record("com.example.legacy-ui", super::types::ExtensionKind::Local);
    legacy.manifest.ui_legacy = Some("../../obsolete/ui.tsx".to_string());
    legacy.contributions.tools.push(ExtensionTool {
        name: "com.example.legacy-ui.healthy".to_string(),
        description: "Healthy".to_string(),
        parameters: json!({"type":"object"}),
        effect: ExtensionEffect::ReadOnly,
        replaces_core: false,
    });

    let diagnostics = super::runtime_sync::manifest_ui_diagnostics(std::slice::from_ref(&legacy));

    assert!(legacy.manifest.ui.is_none());
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].extension_id, legacy.manifest.id);
    assert_eq!(diagnostics[0].code, "ui_manifest_legacy");
    assert!(super::validation::manifest(&legacy.manifest).is_ok());
    assert_eq!(legacy.contributions.tools.len(), 1);
}

#[test]
fn missing_or_invalid_tool_effect_is_revalidated_as_unknown() {
    for value in [
        json!({"name":"tool","description":"Tool","parameters":{}}),
        json!({"name":"tool","description":"Tool","parameters":{},"effect":"root"}),
        json!({"name":"tool","description":"Tool","parameters":{},"effect":null}),
        json!({"name":"tool","description":"Tool","parameters":{},"effect":42}),
        json!({"name":"tool","description":"Tool","parameters":{},"effect":[]}),
    ] {
        let tool: ExtensionTool = serde_json::from_value(value).unwrap();
        assert_eq!(tool.effect, ExtensionEffect::Unknown);
    }
}

#[test]
fn stable_extensions_cannot_replace_core_tools() {
    let spec = HostExtensionSpec {
        id: "com.example.stable".to_string(),
        main_path: "/tmp/index.ts".to_string(),
        manifest: ExtensionManifest {
            id: "com.example.stable".to_string(),
            name: "Stable".to_string(),
            version: "1.0.0".to_string(),
            beaver_api: "1".to_string(),
            runtime: "node".to_string(),
            main: Some("index.ts".to_string()),
            ui: None,
            ui_legacy: None,
            access: "full".to_string(),
            api_level: ExtensionApiLevel::Stable,
            essential: false,
            author: None,
            homepage: None,
            description: None,
        },
    };
    let contributions = ExtensionContributions {
        tools: vec![ExtensionTool {
            name: "web_search".to_string(),
            description: "Replacement".to_string(),
            parameters: json!({"type": "object"}),
            effect: ExtensionEffect::Unknown,
            replaces_core: true,
        }],
        events: Vec::new(),
        ui: Vec::new(),
    };

    assert!(!accepts_contributions(&spec, &contributions));
}

#[test]
fn invalid_standard_ui_is_dropped_without_losing_valid_tools() {
    let mut spec = HostExtensionSpec {
        id: "com.example.partial-ui".to_string(),
        main_path: "/tmp/index.ts".to_string(),
        manifest: ExtensionManifest {
            id: "com.example.partial-ui".to_string(),
            name: "Partial UI".to_string(),
            version: "1.0.0".to_string(),
            beaver_api: "1".to_string(),
            runtime: "node".to_string(),
            main: Some("index.ts".to_string()),
            ui: None,
            ui_legacy: None,
            access: "full".to_string(),
            api_level: ExtensionApiLevel::Stable,
            essential: false,
            author: None,
            homepage: None,
            description: None,
        },
    };
    spec.manifest.ui = Some(super::types::ExtensionUiManifest {
        api_version: "1".to_string(),
        mode: super::types::ExtensionUiMode::Standard,
        entry: None,
    });
    let contributions = ExtensionContributions {
        tools: vec![ExtensionTool {
            name: "com.example.partial-ui.healthy".to_string(),
            description: "Healthy".to_string(),
            parameters: json!({"type":"object"}),
            effect: ExtensionEffect::ReadOnly,
            replaces_core: false,
        }],
        events: Vec::new(),
        ui: vec![json!({
            "type":"action", "id":"broken", "placement":"app.toolbar.primary",
            "order":0, "label":{"default":"x".repeat(super::ui_contract::MAX_TEXT_CHARS + 1)},
            "actionId":"broken"
        })],
    };

    let validated = super::runtime_sync_contributions::validate(
        &super::host_identity::HostIdentity::ThirdParty(spec.id.clone()),
        &spec.id,
        &spec,
        contributions,
    )
    .unwrap();
    assert_eq!(validated.core.tools.len(), 1);
    assert!(validated.ui.is_empty());
    assert_eq!(
        validated.ui_diagnostic.as_deref(),
        Some("ui_contribution_invalid")
    );
}

#[test]
fn interrupted_extension_is_excluded_but_other_records_continue() {
    let mut records = vec![
        record("beaver.builtin", super::types::ExtensionKind::Builtin),
        record("com.example.crash", super::types::ExtensionKind::Local),
        record("com.example.safe", super::types::ExtensionKind::Local),
    ];

    super::registry_interruption::mark_interrupted_records(&mut records, "com.example.crash");
    let interrupted = &records[1];
    assert!(interrupted.enabled);
    assert_eq!(interrupted.status, super::types::ExtensionStatus::Error);
    assert_eq!(
        interrupted.last_error.as_deref(),
        Some(super::error_codes::LOAD_INTERRUPTED)
    );

    let eligible = super::runtime_sync::filter_for_recovery(
        records,
        &super::runtime_sync::RecoveryPreflight::Interrupted("com.example.crash".to_string()),
    );

    assert_eq!(
        eligible
            .iter()
            .map(|record| record.manifest.id.as_str())
            .collect::<Vec<_>>(),
        vec!["beaver.builtin", "com.example.safe"]
    );
}

#[test]
fn invalid_marker_temporarily_excludes_only_local_records() {
    let records = vec![
        record("beaver.builtin", super::types::ExtensionKind::Builtin),
        record("com.example.local", super::types::ExtensionKind::Local),
    ];

    let eligible = super::runtime_sync::filter_for_recovery(
        records,
        &super::runtime_sync::RecoveryPreflight::Invalid,
    );

    assert_eq!(eligible.len(), 1);
    assert_eq!(eligible[0].manifest.id, "beaver.builtin");
    assert!(eligible[0].enabled);
}

#[test]
fn cautious_retry_reloads_other_enabled_hosts_without_granting_them_retry_attempts() {
    let records = vec![
        record("beaver.word", super::types::ExtensionKind::Builtin),
        record("com.example.crash", super::types::ExtensionKind::Local),
        record("com.example.other", super::types::ExtensionKind::Local),
    ];

    let eligible = super::runtime_sync::filter_for_recovery(
        records,
        &super::runtime_sync::RecoveryPreflight::Retry("com.example.crash".to_string(), 2),
    );

    assert_eq!(eligible.len(), 3);
    assert_eq!(
        eligible
            .iter()
            .map(|record| record.manifest.id.as_str())
            .collect::<Vec<_>>(),
        vec!["beaver.word", "com.example.crash", "com.example.other"]
    );
    let recovery =
        super::runtime_sync::RecoveryPreflight::Retry("com.example.crash".to_string(), 2);
    assert_eq!(recovery.attempts_for("com.example.crash"), 2);
    assert_eq!(recovery.attempts_for("com.example.other"), 1);
}

#[test]
fn builtin_retry_keeps_every_enabled_host_coherent_after_the_shared_reset() {
    let records = vec![
        record("beaver.word", super::types::ExtensionKind::Builtin),
        record("beaver.excel", super::types::ExtensionKind::Builtin),
        record("com.example.local", super::types::ExtensionKind::Local),
    ];

    let eligible = super::runtime_sync::filter_for_recovery(
        records,
        &super::runtime_sync::RecoveryPreflight::Retry("beaver.word".to_string(), 2),
    );

    assert_eq!(
        eligible
            .iter()
            .map(|record| record.manifest.id.as_str())
            .collect::<Vec<_>>(),
        vec!["beaver.word", "beaver.excel", "com.example.local"]
    );
}

#[test]
fn orphaned_interruption_is_treated_as_invalid_instead_of_loading_all_locals() {
    let records = vec![
        record("beaver.word", super::types::ExtensionKind::Builtin),
        record("com.example.local", super::types::ExtensionKind::Local),
    ];

    let recovery =
        super::runtime_sync::RecoveryPreflight::Interrupted("com.example.removed".to_string())
            .resolve_for(&records)
            .unwrap();
    let eligible = super::runtime_sync::filter_for_recovery(records, &recovery);

    assert!(matches!(
        recovery,
        super::runtime_sync::RecoveryPreflight::Invalid
    ));
    assert_eq!(eligible.len(), 1);
    assert_eq!(eligible[0].manifest.id, "beaver.word");
}

#[test]
fn an_installed_but_disabled_interruption_stays_attributed_without_blocking_neighbors() {
    let mut interrupted = record(
        "com.example.interrupted",
        super::types::ExtensionKind::Local,
    );
    interrupted.enabled = false;
    interrupted.trusted = false;
    let all_records = vec![
        interrupted,
        record("com.example.neighbor", super::types::ExtensionKind::Local),
    ];

    let recovery =
        super::runtime_sync::RecoveryPreflight::Interrupted("com.example.interrupted".to_string())
            .resolve_for(&all_records)
            .unwrap();
    assert!(matches!(
        recovery,
        super::runtime_sync::RecoveryPreflight::Interrupted(_)
    ));

    let eligible = all_records
        .into_iter()
        .filter(|record| record.enabled && record.trusted)
        .collect();
    let filtered = super::runtime_sync::filter_for_recovery(eligible, &recovery);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].manifest.id, "com.example.neighbor");
}

#[test]
fn retry_refuses_a_marker_target_that_is_no_longer_enabled_and_trusted() {
    let records = vec![record(
        "com.example.other",
        super::types::ExtensionKind::Local,
    )];

    assert!(
        super::runtime_sync::RecoveryPreflight::Retry("com.example.removed".to_string(), 2,)
            .resolve_for(&records)
            .is_err()
    );
}

#[test]
fn retry_authority_is_bound_to_the_current_marker_generation() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("extension-loading.json");
    super::loading_marker::start_at(&path, "com.example.crash", 1).unwrap();
    let marker = super::loading_marker::read_at(&path);

    assert!(
        super::runtime_sync::RecoveryPreflight::Retry("com.example.crash".to_string(), 2,)
            .validate_retry_marker(&marker)
            .is_ok()
    );
    assert!(
        super::runtime_sync::RecoveryPreflight::Retry("com.example.other".to_string(), 2,)
            .validate_retry_marker(&marker)
            .is_err()
    );
    assert!(
        super::runtime_sync::RecoveryPreflight::Retry("com.example.crash".to_string(), 3,)
            .validate_retry_marker(&marker)
            .is_err()
    );
}
