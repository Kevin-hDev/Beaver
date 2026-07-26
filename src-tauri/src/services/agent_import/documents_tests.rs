use super::*;
use crate::services::agent_import::discovery::scan_source;
use crate::services::agent_import::source_specs::source_specs_with;
use tempfile::TempDir;

pub(super) fn prepare() -> (TempDir, std::path::PathBuf, SourceSelection) {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("home");
    let data = temp.path().join("data");
    let source = home.join(".claude/CLAUDE.md");
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(&source, b"# Claude\nKeep this exact.\n").unwrap();
    let spec = source_specs_with(&home, &home.join(".config"), &home.join(".kimi-code"))
        .into_iter()
        .find(|spec| spec.id == "claude")
        .unwrap();
    let found = scan_source(&spec, &home, &AgentImportRegistry::default());
    let selection = SourceSelection {
        source_id: "claude".into(),
        enabled: true,
        skill_mode: SelectionMode::All,
        selected_skill_ids: Vec::new(),
        selected_rule_ids: Vec::new(),
        selected_document_ids: vec![found.documents[0].public.id.clone()],
    };
    (temp, data, selection)
}

#[test]
fn imports_document_verbatim_and_saves_registry() {
    let (temp, data, selection) = prepare();
    let home = temp.path().join("home");

    let result = save_source_selection_to(&home, &data, selection, false).unwrap();

    assert!(result.saved);
    assert_eq!(
        std::fs::read(data.join("CLAUDE.md")).unwrap(),
        b"# Claude\nKeep this exact.\n"
    );
    assert_eq!(
        registry::read_from(&data.join("external-agent-sources.json"))
            .sources
            .len(),
        1
    );
}

#[test]
fn existing_document_requires_explicit_replacement() {
    let (temp, data, selection) = prepare();
    let home = temp.path().join("home");
    std::fs::write(data.join("CLAUDE.md"), b"# Local").unwrap();

    let result = save_source_selection_to(&home, &data, selection, false).unwrap();

    assert!(!result.saved);
    assert_eq!(result.conflicts, vec!["CLAUDE.md"]);
    assert_eq!(std::fs::read(data.join("CLAUDE.md")).unwrap(), b"# Local");
}

#[test]
fn confirmed_replacement_creates_backup() {
    let (temp, data, selection) = prepare();
    let home = temp.path().join("home");
    std::fs::write(data.join("CLAUDE.md"), b"# Local").unwrap();

    let result = save_source_selection_to(&home, &data, selection, true).unwrap();

    assert!(result.saved);
    let backup_count = std::fs::read_dir(data.join("agent-import-backups"))
        .unwrap()
        .count();
    assert_eq!(backup_count, 1);
}

#[test]
fn invalid_utf8_document_is_rejected() {
    let (temp, data, selection) = prepare();
    let home = temp.path().join("home");
    std::fs::write(home.join(".claude/CLAUDE.md"), [0xff, 0xfe]).unwrap();

    assert!(save_source_selection_to(&home, &data, selection, false).is_err());
    assert!(!data.join("CLAUDE.md").exists());
}

#[test]
fn disabling_source_preserves_selection_and_imported_document() {
    let (temp, data, selection) = prepare();
    let home = temp.path().join("home");
    save_source_selection_to(&home, &data, selection.clone(), false).unwrap();
    let mut disabled = selection;
    disabled.enabled = false;
    disabled.selected_document_ids.clear();

    save_source_selection_to(&home, &data, disabled, false).unwrap();

    let registry = registry::read_from(&data.join("external-agent-sources.json"));
    assert!(!registry.sources[0].enabled);
    assert_eq!(registry.sources[0].selected_document_ids.len(), 1);
    assert!(registry.documents[0].enabled);
}

#[test]
fn disabling_one_source_does_not_disable_another_source_document() {
    let (temp, data, selection) = prepare();
    let home = temp.path().join("home");
    save_source_selection_to(&home, &data, selection.clone(), false).unwrap();
    let registry_path = data.join("external-agent-sources.json");
    let mut registry = registry::read_from(&registry_path);
    registry.sources.push(SourceSelection {
        source_id: "qwen".into(),
        enabled: true,
        skill_mode: SelectionMode::None,
        selected_skill_ids: Vec::new(),
        selected_rule_ids: Vec::new(),
        selected_document_ids: Vec::new(),
    });
    registry.documents.push(ImportedDocument {
        name: "QWEN.md".into(),
        source_id: "qwen".into(),
        source_path: home.join(".qwen/QWEN.md"),
        source_hash: "0".repeat(64),
        enabled: true,
    });
    registry::write_to(&registry_path, &registry).unwrap();
    let mut disabled = selection;
    disabled.enabled = false;

    save_source_selection_to(&home, &data, disabled, false).unwrap();

    let loaded = registry::read_from(&registry_path);
    assert!(
        loaded
            .documents
            .iter()
            .find(|document| document.name == "QWEN.md")
            .unwrap()
            .enabled
    );
}

#[test]
fn changed_source_document_is_reported_without_automatic_sync() {
    let (temp, data, selection) = prepare();
    let home = temp.path().join("home");
    save_source_selection_to(&home, &data, selection, false).unwrap();
    std::fs::write(home.join(".claude/CLAUDE.md"), "# New source").unwrap();
    let registry = registry::read_from(&data.join("external-agent-sources.json"));
    let spec = source_specs_with(&home, &home.join(".config"), &home.join(".kimi-code"))
        .into_iter()
        .find(|spec| spec.id == "claude")
        .unwrap();

    let scanned = scan_source(&spec, &home, &registry);

    assert!(scanned.summary.documents[0].update_available);
    assert_eq!(
        std::fs::read_to_string(data.join("CLAUDE.md")).unwrap(),
        "# Claude\nKeep this exact.\n"
    );
}

#[test]
fn document_backups_are_bounded_to_five() {
    let (temp, data, selection) = prepare();
    let home = temp.path().join("home");
    save_source_selection_to(&home, &data, selection.clone(), false).unwrap();

    for index in 0..7 {
        std::fs::write(
            home.join(".claude/CLAUDE.md"),
            format!("# Revision {index}"),
        )
        .unwrap();
        save_source_selection_to(&home, &data, selection.clone(), true).unwrap();
    }

    assert_eq!(
        std::fs::read_dir(data.join("agent-import-backups"))
            .unwrap()
            .count(),
        5
    );
}
