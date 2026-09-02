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
    };

    assert!(!accepts_contributions(&spec, &contributions));
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
