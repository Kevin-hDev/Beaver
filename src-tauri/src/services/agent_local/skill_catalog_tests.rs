use super::*;

#[test]
fn command_name_is_qualified_by_source() {
    let command = command_name(
        "claude",
        "frontend-design",
        "claude:skill:0123456789abcdef",
    );

    assert_eq!(command, "claude:frontend-design");
}

#[test]
fn local_command_keeps_legacy_unqualified_name() {
    let command = command_name("local", "frontend-design", "local:skill:0123456789");

    assert_eq!(command, "frontend-design");
}

#[test]
fn local_skills_use_the_beaver_source_name() {
    assert_eq!(local_source_name(), "Beaver");
}

#[test]
fn command_name_filters_unsafe_characters() {
    let command = command_name("agents", "review / ../ secrets", "agents:skill:123456789012");

    assert_eq!(command, "agents:reviewsecrets");
    assert!(!command.contains('/'));
}

#[test]
fn catalog_id_does_not_expose_path() {
    let id = catalog_id("local", Path::new("/private/example/skill"));

    assert!(id.starts_with("local:skill:"));
    assert!(!id.contains("private"));
}

#[test]
fn duplicate_commands_receive_stable_id_suffixes() {
    let make_entry = |id: &str| SkillCatalogEntry {
        info: SkillInfo {
            id: id.to_string(),
            name: "review".into(),
            command: "claude:review".into(),
            description: String::new(),
            path: id.to_string(),
            source: "claude".into(),
            source_name: "Claude Code".into(),
        },
        manifest: PathBuf::from("SKILL.md"),
        bundle_root: PathBuf::from("."),
    };
    let mut entries = vec![
        make_entry("claude:skill:11111111"),
        make_entry("claude:skill:22222222"),
    ];

    make_commands_unique(&mut entries);

    assert_eq!(entries[0].info.command, "claude:review:11111111");
    assert_eq!(entries[1].info.command, "claude:review:22222222");
}

#[test]
fn metadata_keeps_up_to_250_unicode_characters() {
    let temp = tempfile::TempDir::new().unwrap();
    let manifest = temp.path().join("SKILL.md");
    let description = "é".repeat(MAX_SKILL_DESCRIPTION_CHARS + 20);
    std::fs::write(
        &manifest,
        format!("---\nname: bounded\ndescription: {description}\n---\nBody"),
    )
    .unwrap();

    let (_, bounded) = metadata(&manifest, "fallback").unwrap();

    assert_eq!(bounded.chars().count(), MAX_SKILL_DESCRIPTION_CHARS);
    assert_eq!(bounded, "é".repeat(MAX_SKILL_DESCRIPTION_CHARS));
}
#[test]
fn catalog_reserves_extension_qualified_ids() {
    assert!(!super::valid_catalog_id("extension:plugin:guide"));
    assert!(super::valid_catalog_id("local:skill:guide"));
}

#[test]
fn global_catalog_filter_removes_extension_qualified_entries() {
    let make_entry = |id: &str| SkillCatalogEntry {
        info: SkillInfo {
            id: id.into(),
            name: "guide".into(),
            command: "guide".into(),
            description: String::new(),
            path: id.into(),
            source: "test".into(),
            source_name: "Test".into(),
        },
        manifest: PathBuf::from("SKILL.md"),
        bundle_root: PathBuf::from("."),
    };
    let mut entries = vec![
        make_entry("local:skill:guide"),
        make_entry("extension:example.plugin:guide"),
    ];

    retain_global_entries(&mut entries);

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].info.id, "local:skill:guide");
}
