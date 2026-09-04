use super::super::types::{
    ExtensionApiLevel, ExtensionContributions, ExtensionKind, ExtensionManifest, ExtensionRecord,
    ExtensionSkill, ExtensionStatus,
};
use super::*;

fn record_with_skill() -> ExtensionRecord {
    ExtensionRecord {
        manifest: ExtensionManifest {
            id: "com.example.skills".to_string(),
            name: "Skills".to_string(),
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
            description: Some("Skill-only extension".to_string()),
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
        status: ExtensionStatus::Active,
        last_error: None,
        last_activated_at: None,
        sensitive_access_granted: false,
        contributions: ExtensionContributions {
            skills: vec![ExtensionSkill {
                id: "guide".to_string(),
                name: "Guide".to_string(),
                description: "Description.".to_string(),
                path: "SKILL.md".to_string(),
            }],
            ..Default::default()
        },
    }
}

fn snapshot(version: &str, text: &str, capacity_plugin_ids: &[&str]) -> CatalogSnapshot {
    CatalogSnapshot {
        text: text.to_string(),
        version: version.to_string(),
        ordered_plugin_ids: Vec::new(),
        capacity_plugin_ids: capacity_plugin_ids
            .iter()
            .map(|id| (*id).to_string())
            .collect(),
        protected_plugin_ids: Vec::new(),
        essential_plugin_ids: Vec::new(),
    }
}

#[test]
fn unchanged_registry_version_preserves_catalog_and_capacity_order() {
    let previous = snapshot("same", "stable", &["example.alpha"]);
    let next = snapshot("same", "changed", &["example.frequent"]);

    let selected = stable_catalog(previous.clone(), next);

    assert_eq!(selected.text, previous.text);
    assert_eq!(selected.capacity_plugin_ids, previous.capacity_plugin_ids);
}

#[test]
fn changed_registry_version_accepts_the_new_catalog() {
    let previous = snapshot("old", "stable", &["example.alpha"]);
    let next = snapshot("new", "updated", &["example.frequent"]);

    let selected = stable_catalog(previous, next.clone());

    assert_eq!(selected.text, next.text);
    assert_eq!(selected.capacity_plugin_ids, next.capacity_plugin_ids);
}

#[test]
fn unavailable_usage_scores_never_block_registry_rebuilds() {
    let scores = usage_scores_with(|| Err("unavailable".to_string()));

    assert!(scores.is_empty());
}

#[test]
fn skill_only_extension_remains_indexed_without_a_tool() {
    let plugins = plugins_from_records(&[record_with_skill()]);

    assert_eq!(plugins.len(), 1);
    assert!(plugins[0].tools.is_empty());
    assert_eq!(plugins[0].skills[0].id, "guide");
}

#[test]
fn identical_local_skill_ids_remain_attributed_to_their_extensions() {
    let first = record_with_skill();
    let mut second = record_with_skill();
    second.manifest.id = "com.example.other-skills".to_string();
    second.manifest.name = "Other skills".to_string();

    let plugins = plugins_from_records(&[first, second]);

    assert_eq!(plugins.len(), 2);
    assert_eq!(plugins[0].id, "com.example.skills");
    assert_eq!(plugins[0].skills[0].id, "guide");
    assert_eq!(plugins[1].id, "com.example.other-skills");
    assert_eq!(plugins[1].skills[0].id, "guide");
}
