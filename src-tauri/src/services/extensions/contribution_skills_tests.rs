#[test]
fn contribution_paths_reject_absolute_parent_control_and_windows_forms() {
    for path in [
        "/absolute",
        "../parent",
        "dir/../parent",
        "dir\\file",
        "C:/file",
        "dir\0file",
        "dir\nfile",
    ] {
        assert!(
            super::contribution_path::validate(path).is_err(),
            "{path:?}"
        );
    }
    assert!(super::contribution_path::validate("skills/guide.md").is_ok());
}

#[test]
fn contribution_paths_reject_ads_and_all_dos_reserved_name_variants() {
    for path in [
        "resources/file.txt:stream",
        "CON",
        "con.txt",
        "PRN .md",
        "aux...",
        "NUL ",
        "com1.txt",
        "COM9... ",
        "lpt1.md",
        "LPT9 .txt",
    ] {
        assert!(
            super::contribution_path::validate(path).is_err(),
            "{path:?} must be refused"
        );
    }
}

#[test]
fn skills_reject_duplicate_ids_and_overlong_human_metadata() {
    let skill = super::types::ExtensionSkill {
        id: "guide".to_string(),
        name: "Guide".to_string(),
        description: "Description.".to_string(),
        path: "skills/guide/SKILL.md".to_string(),
    };
    assert!(super::contribution_skills::validate(&[skill.clone(), skill.clone()]).is_err());

    let mut overlong = skill;
    overlong.name = "🦫".repeat(super::types::MAX_EXTENSION_NAME_CHARS + 1);
    assert!(super::contribution_skills::validate(&[overlong]).is_err());
}

#[test]
fn skills_enforce_the_generated_collection_limit() {
    let skills = (0..=super::types::MAX_SKILLS_PER_EXTENSION)
        .map(|index| super::types::ExtensionSkill {
            id: format!("skill-{index}"),
            name: "Skill".to_string(),
            description: "Description.".to_string(),
            path: "skills/guide/SKILL.md".to_string(),
        })
        .collect::<Vec<_>>();

    assert!(super::contribution_skills::validate(&skills).is_err());
}

#[test]
fn skills_require_a_manifest_filename_and_reject_windows_superscripts() {
    for path in [
        "guide.txt",
        "skills/guide.md",
        "Skill.md",
        "COM¹/SKILL.md",
        "LPT².txt/SKILL.md",
    ] {
        let skill = super::types::ExtensionSkill {
            id: "guide".into(),
            name: "Guide".into(),
            description: "Guide".into(),
            path: path.into(),
        };
        assert!(
            super::contribution_skills::validate(&[skill]).is_err(),
            "{path}"
        );
    }
    for path in ["SKILL.md", "skills/skill.md"] {
        let skill = super::types::ExtensionSkill {
            id: "guide".into(),
            name: "Guide".into(),
            description: "Guide".into(),
            path: path.into(),
        };
        assert!(super::contribution_skills::validate(&[skill]).is_ok());
    }
}
