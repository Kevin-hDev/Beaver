use super::*;
use crate::services::agent_import::models::SelectionMode;
use tempfile::TempDir;

fn selection(source_id: &str) -> SourceSelection {
    SourceSelection {
        source_id: source_id.to_string(),
        enabled: true,
        skill_mode: SelectionMode::All,
        selected_skill_ids: Vec::new(),
        selected_rule_ids: vec![format!("{source_id}:rules/example.md")],
        selected_document_ids: Vec::new(),
    }
}

#[test]
fn registry_round_trip_keeps_selection() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("registry.json");
    let mut registry = AgentImportRegistry::default();
    upsert_source(&mut registry, selection("claude")).unwrap();

    write_to(&path, &registry).unwrap();

    let loaded = read_from(&path);
    assert_eq!(loaded.sources.len(), 1);
    assert_eq!(loaded.sources[0].source_id, "claude");
}

#[test]
fn upsert_replaces_existing_source_without_growing() {
    let mut registry = AgentImportRegistry::default();
    upsert_source(&mut registry, selection("codex")).unwrap();
    let mut changed = selection("codex");
    changed.enabled = false;

    upsert_source(&mut registry, changed).unwrap();

    assert_eq!(registry.sources.len(), 1);
    assert!(!registry.sources[0].enabled);
}

#[test]
fn invalid_source_is_rejected() {
    let mut registry = AgentImportRegistry::default();
    assert!(upsert_source(&mut registry, selection("unknown")).is_err());
}

#[test]
fn traversal_in_item_id_is_rejected() {
    let mut invalid = selection("agents");
    invalid.selected_rule_ids = vec!["agents:../secret".to_string()];
    let mut registry = AgentImportRegistry::default();

    assert!(upsert_source(&mut registry, invalid).is_err());
}

#[test]
fn oversized_registry_is_ignored() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("registry.json");
    std::fs::write(&path, vec![b'x'; MAX_REGISTRY_BYTES as usize + 1]).unwrap();

    assert!(read_from(&path).sources.is_empty());
}

#[test]
fn duplicate_item_ids_are_rejected() {
    let mut invalid = selection("agents");
    invalid.selected_rule_ids = vec!["agents:rule:one".into(), "agents:rule:one".into()];
    let mut registry = AgentImportRegistry::default();

    assert!(upsert_source(&mut registry, invalid).is_err());
}

#[test]
fn relative_document_source_path_is_rejected() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("registry.json");
    let registry = AgentImportRegistry {
        version: 1,
        sources: vec![selection("claude")],
        documents: vec![ImportedDocument {
            name: "CLAUDE.md".into(),
            source_id: "claude".into(),
            source_path: "relative/CLAUDE.md".into(),
            source_hash: "0".repeat(64),
            enabled: true,
        }],
    };

    assert!(write_to(&path, &registry).is_err());
}
