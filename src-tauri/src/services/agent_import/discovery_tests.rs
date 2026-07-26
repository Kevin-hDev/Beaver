use super::*;
use crate::services::agent_import::models::{SelectionMode, SourceSelection};
use crate::services::agent_import::source_specs::source_specs_with;
use tempfile::TempDir;

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn claude_spec(home: &Path) -> SourceSpec {
    source_specs_with(home, &home.join(".config"), &home.join(".kimi-code"))
        .into_iter()
        .find(|spec| spec.id == "claude")
        .unwrap()
}

#[test]
fn source_catalog_contains_nine_independent_sources() {
    let temp = TempDir::new().unwrap();
    let specs = source_specs_with(
        temp.path(),
        &temp.path().join(".config"),
        &temp.path().join(".kimi-code"),
    );

    assert_eq!(specs.len(), 9);
    assert_eq!(
        specs.iter().map(|spec| spec.id).collect::<Vec<_>>(),
        vec![
            "claude", "codex", "agents", "hermes", "qwen", "zcode", "openclaw", "opencode", "kimi"
        ]
    );
}

#[test]
fn claude_discovers_document_rule_and_recursive_skill() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join(".claude");
    write(&root.join("CLAUDE.md"), "# Instructions");
    write(&root.join("rules/style.md"), "Use short sentences.");
    write(&root.join("rules/nested/tests.md"), "Write tests.");
    write(
        &root.join("skills/group/review/SKILL.md"),
        "---\nname: review\ndescription: Review changes\n---\nDo it.",
    );

    let source = scan_source(
        &claude_spec(temp.path()),
        temp.path(),
        &AgentImportRegistry::default(),
    );

    assert_eq!(source.summary.status, SourceStatus::Detected);
    assert_eq!(source.documents.len(), 1);
    assert_eq!(source.rules.len(), 2);
    assert_eq!(source.skills.len(), 1);
    assert!(source.summary.skills[0].selected);
    assert_eq!(source.summary.skills[0].name, "review");
}

#[test]
fn custom_selection_does_not_enable_new_skill() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join(".claude");
    write(
        &root.join("skills/review/SKILL.md"),
        "---\nname: review\n---\nReview.",
    );
    let spec = claude_spec(temp.path());
    let first = scan_source(&spec, temp.path(), &AgentImportRegistry::default());
    let selected_id = first.skills[0].public.id.clone();
    write(
        &root.join("skills/new-skill/SKILL.md"),
        "---\nname: new-skill\n---\nNew.",
    );
    let registry = AgentImportRegistry {
        version: 1,
        sources: vec![SourceSelection {
            source_id: "claude".into(),
            enabled: true,
            skill_mode: SelectionMode::Custom,
            selected_skill_ids: vec![selected_id],
            selected_rule_ids: Vec::new(),
            selected_document_ids: Vec::new(),
        }],
        documents: Vec::new(),
    };

    let rescanned = scan_source(&spec, temp.path(), &registry);

    assert_eq!(rescanned.skills.len(), 2);
    assert_eq!(
        rescanned
            .summary
            .skills
            .iter()
            .filter(|skill| skill.selected)
            .count(),
        1
    );
}

#[test]
fn excluded_identity_document_is_not_a_rule() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join(".claude");
    write(&root.join("rules/SOUL.md"), "Identity");
    write(&root.join("rules/safe.md"), "Rule");

    let source = scan_source(
        &claude_spec(temp.path()),
        temp.path(),
        &AgentImportRegistry::default(),
    );

    assert_eq!(source.rules.len(), 1);
    assert_eq!(source.rules[0].public.name, "safe.md");
}

#[test]
fn installed_source_without_content_is_empty() {
    let temp = TempDir::new().unwrap();
    std::fs::create_dir_all(temp.path().join(".claude")).unwrap();

    let source = scan_source(
        &claude_spec(temp.path()),
        temp.path(),
        &AgentImportRegistry::default(),
    );

    assert_eq!(source.summary.status, SourceStatus::Empty);
}

#[test]
fn all_and_none_modes_control_new_skills() {
    let temp = TempDir::new().unwrap();
    write(
        &temp.path().join(".claude/skills/review/SKILL.md"),
        "---\nname: review\n---\nReview.",
    );
    let spec = claude_spec(temp.path());
    let selection = |mode| SourceSelection {
        source_id: "claude".into(),
        enabled: true,
        skill_mode: mode,
        selected_skill_ids: Vec::new(),
        selected_rule_ids: Vec::new(),
        selected_document_ids: Vec::new(),
    };
    let registry = |mode| AgentImportRegistry {
        version: 1,
        sources: vec![selection(mode)],
        documents: Vec::new(),
    };

    let all = scan_source(&spec, temp.path(), &registry(SelectionMode::All));
    let none = scan_source(&spec, temp.path(), &registry(SelectionMode::None));

    assert!(all.summary.skills[0].selected);
    assert!(!none.summary.skills[0].selected);
}
