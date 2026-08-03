use super::*;

fn test_spec() -> SourceSpec {
    SourceSpec {
        id: "test",
        display_name: "Test",
        detection_roots: Vec::new(),
        documents: Vec::new(),
        rule_roots: Vec::new(),
        skill_roots: Vec::new(),
    }
}

#[test]
fn imported_skill_descriptions_keep_up_to_250_unicode_characters() {
    let description = "é".repeat(MAX_SKILL_DESCRIPTION_CHARS + 20);

    let item = import_item(
        &test_spec(),
        Path::new("skill"),
        ImportItemKind::Skill,
        "skill".to_string(),
        description,
    );

    assert_eq!(
        item.description.chars().count(),
        MAX_SKILL_DESCRIPTION_CHARS
    );
    assert_eq!(item.description, "é".repeat(MAX_SKILL_DESCRIPTION_CHARS));
}
