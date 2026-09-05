use super::registry_sync::{apply_all_enabled_error, apply_loaded_results, mark_loading_records};
use super::types::{
    ExtensionApiLevel, ExtensionContributions, ExtensionEffect, ExtensionKind, ExtensionManifest,
    ExtensionRecord, ExtensionSkill, ExtensionStatus, ExtensionTool,
};
use serde_json::json;
use std::collections::{BTreeMap, HashMap, HashSet};

fn contributions(name: &str) -> ExtensionContributions {
    ExtensionContributions {
        tools: vec![ExtensionTool {
            name: name.to_string(),
            description: "Tool".to_string(),
            parameters: json!({"type": "object"}),
            effect: ExtensionEffect::ReadOnly,
            replaces_core: false,
        }],
        events: Vec::new(),
        ui: Vec::new(),
        ..Default::default()
    }
}

fn record(id: &str) -> ExtensionRecord {
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
        enabled: true,
        trusted: true,
        fingerprint: None,
        ui_artifact: None,
        trusted_at: None,
        show_in_chat: true,
        status: ExtensionStatus::Loading,
        last_error: None,
        last_activated_at: None,
        sensitive_access_granted: false,
        contributions: ExtensionContributions::default(),
    }
}

#[test]
fn unavailable_host_uses_the_canonical_last_error_code() {
    let mut records = vec![record("com.example.enabled")];
    records[0].enabled = true;
    apply_all_enabled_error(&mut records);
    assert_eq!(
        records[0].last_error.as_deref(),
        Some(super::error_codes::HOST_UNAVAILABLE)
    );
}

#[test]
fn second_plugin_with_the_same_canonical_tool_name_is_rejected() {
    let successful = HashMap::from([
        ("plugin-a".to_string(), contributions("shared.tool")),
        ("plugin-b".to_string(), contributions("shared.tool")),
    ]);
    let enabled = HashSet::from(["plugin-a".to_string(), "plugin-b".to_string()]);
    let mut records = vec![record("plugin-a"), record("plugin-b")];
    let mut active = 0;
    apply_loaded_results(
        &mut records,
        &enabled,
        &successful,
        &BTreeMap::new(),
        &mut active,
    );
    assert_eq!(active, 1);
    assert_eq!(records[0].status, ExtensionStatus::Active);
    assert_eq!(records[1].status, ExtensionStatus::Error);
    assert_eq!(records[1].last_error.as_deref(), Some("load_failed"));
    assert!(records[1].contributions.tools.is_empty());
}

#[test]
fn loading_transition_only_clears_errors_for_eligible_records() {
    let mut eligible = record("eligible");
    eligible.status = ExtensionStatus::Error;
    eligible.last_error = Some("previous".to_string());
    let mut revoked = record("revoked");
    revoked.status = ExtensionStatus::Error;
    revoked.last_error = Some("extensions_fingerprint_changed".to_string());
    let mut records = vec![eligible, revoked];
    mark_loading_records(&mut records, &HashSet::from(["eligible".to_string()]));
    assert_eq!(records[0].status, ExtensionStatus::Loading);
    assert!(records[0].last_error.is_none());
    assert_eq!(records[1].status, ExtensionStatus::Error);
    assert_eq!(
        records[1].last_error.as_deref(),
        Some("extensions_fingerprint_changed")
    );
}

#[test]
fn restarted_host_reconstructs_a_skill_only_extension() {
    let successful = HashMap::from([(
        "plugin-a".to_string(),
        ExtensionContributions {
            skills: vec![ExtensionSkill {
                id: "guide".to_string(),
                name: "Guide".to_string(),
                description: "Description.".to_string(),
                path: "skills/guide/SKILL.md".to_string(),
            }],
            ..Default::default()
        },
    )]);
    let mut records = vec![record("plugin-a")];
    let mut active = 0;

    apply_loaded_results(
        &mut records,
        &HashSet::from(["plugin-a".to_string()]),
        &successful,
        &BTreeMap::new(),
        &mut active,
    );

    assert_eq!(active, 1);
    assert!(records[0].contributions.tools.is_empty());
    assert_eq!(records[0].contributions.skills[0].id, "guide");
}
