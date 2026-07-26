use super::*;
use crate::services::agent_import::models::{SelectionMode, SourceSelection};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap().filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.push((
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    std::fs::read(path).unwrap(),
                ));
            }
        }
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

#[test]
fn configuring_agents_never_modifies_external_source() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("home");
    let data = temp.path().join("data");
    let root = home.join(".agents");
    write(&root.join("AGENTS.md"), "# Shared instructions");
    write(&root.join("rules/style.md"), "Be concise.");
    write(
        &root.join("skills/review/SKILL.md"),
        "---\nname: review\ndescription: Review changes\n---\nReview.",
    );
    write(&root.join("skills/review/template.md"), "Template");
    write(&root.join("rules/SOUL.md"), "Excluded");
    std::fs::create_dir_all(&data).unwrap();
    let before = snapshot(&root);
    let source = discovery::scan_sources(&home, &registry::AgentImportRegistry::default())
        .into_iter()
        .find(|source| source.summary.id == "agents")
        .unwrap();
    let selection = SourceSelection {
        source_id: "agents".into(),
        enabled: true,
        skill_mode: SelectionMode::All,
        selected_skill_ids: source
            .skills
            .iter()
            .map(|item| item.public.id.clone())
            .collect(),
        selected_rule_ids: source
            .rules
            .iter()
            .map(|item| item.public.id.clone())
            .collect(),
        selected_document_ids: source
            .documents
            .iter()
            .map(|item| item.public.id.clone())
            .collect(),
    };

    documents::save_source_selection_to(&home, &data, selection, false).unwrap();

    assert_eq!(snapshot(&root), before);
    assert!(data.join("AGENTS.md").is_file());
    assert!(!data.join("skills").exists());
    assert!(!data.join("rules").exists());
}

#[test]
fn runtime_scans_only_enabled_sources() {
    let temp = TempDir::new().unwrap();
    let home = temp.path();
    write(&home.join(".agents/skills/shared/SKILL.md"), "# Shared");
    write(&home.join(".claude/skills/disabled/SKILL.md"), "# Disabled");
    let registry = registry::AgentImportRegistry {
        version: 1,
        sources: vec![
            SourceSelection {
                source_id: "agents".into(),
                enabled: true,
                skill_mode: SelectionMode::All,
                selected_skill_ids: Vec::new(),
                selected_rule_ids: Vec::new(),
                selected_document_ids: Vec::new(),
            },
            SourceSelection {
                source_id: "claude".into(),
                enabled: false,
                skill_mode: SelectionMode::All,
                selected_skill_ids: Vec::new(),
                selected_rule_ids: Vec::new(),
                selected_document_ids: Vec::new(),
            },
        ],
        documents: Vec::new(),
    };

    let sources = selected_sources_from(home, &registry);

    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0].summary.id, "agents");
}
