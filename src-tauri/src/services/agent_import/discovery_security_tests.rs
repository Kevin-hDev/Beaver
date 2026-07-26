use super::*;
use crate::services::agent_import::source_specs::source_specs_with;
use tempfile::TempDir;

fn claude_spec(home: &Path) -> SourceSpec {
    source_specs_with(home, &home.join(".config"), &home.join(".kimi-code"))
        .into_iter()
        .find(|spec| spec.id == "claude")
        .unwrap()
}

fn write_skill(path: &Path, name: &str) {
    std::fs::create_dir_all(path).unwrap();
    std::fs::write(
        path.join("SKILL.md"),
        format!("---\nname: {name}\ndescription: test\n---\nBody"),
    )
    .unwrap();
}

#[test]
fn oversized_skill_is_reported_as_partial() {
    let temp = TempDir::new().unwrap();
    let skill = temp.path().join(".claude/skills/large");
    std::fs::create_dir_all(&skill).unwrap();
    std::fs::write(
        skill.join("SKILL.md"),
        vec![b'x'; super::super::limits::MAX_MANIFEST_BYTES as usize + 1],
    )
    .unwrap();

    let source = scan_source(
        &claude_spec(temp.path()),
        temp.path(),
        &AgentImportRegistry::default(),
    );

    assert!(source.summary.partial);
    assert!(source.skills.is_empty());
}

#[cfg(unix)]
#[test]
fn symlink_outside_source_root_is_not_exposed() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let outside = temp.path().join("outside");
    let skills = temp.path().join(".claude/skills");
    write_skill(&outside, "outside");
    std::fs::create_dir_all(&skills).unwrap();
    symlink(&outside, skills.join("linked")).unwrap();

    let source = scan_source(
        &claude_spec(temp.path()),
        temp.path(),
        &AgentImportRegistry::default(),
    );

    assert!(source.summary.partial);
    assert!(source.skills.is_empty());
}

#[cfg(unix)]
#[test]
fn two_logical_symlinks_keep_distinct_skill_ids() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let skills = temp.path().join(".claude/skills");
    let real = skills.join("real");
    write_skill(&real, "same");
    symlink(&real, skills.join("alias")).unwrap();

    let source = scan_source(
        &claude_spec(temp.path()),
        temp.path(),
        &AgentImportRegistry::default(),
    );

    assert_eq!(source.skills.len(), 2);
    assert_ne!(source.skills[0].public.id, source.skills[1].public.id);
}

#[cfg(unix)]
#[test]
fn document_symlink_outside_source_root_is_rejected() {
    use std::os::unix::fs::symlink;

    let temp = TempDir::new().unwrap();
    let root = temp.path().join(".claude");
    std::fs::create_dir_all(&root).unwrap();
    let outside = temp.path().join("outside.md");
    std::fs::write(&outside, "secret").unwrap();
    symlink(&outside, root.join("CLAUDE.md")).unwrap();

    let source = scan_source(
        &claude_spec(temp.path()),
        temp.path(),
        &AgentImportRegistry::default(),
    );

    assert!(source.summary.partial);
    assert!(source.documents.is_empty());
}
