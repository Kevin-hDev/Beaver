use super::types::{
    ExtensionContributions, ExtensionResource, ExtensionResourceType, ExtensionSkill,
    MAX_RESOURCES_PER_EXTENSION, MAX_SKILLS_PER_EXTENSION,
};

fn contributions() -> ExtensionContributions {
    ExtensionContributions {
        skills: vec![ExtensionSkill {
            id: "reference-skill".to_string(),
            name: "reference-skill".to_string(),
            description: "Une compétence de référence.".to_string(),
            path: "SKILL.md".to_string(),
        }],
        resources: vec![ExtensionResource {
            id: "reference".to_string(),
            name: "reference".to_string(),
            description: "Une ressource texte.".to_string(),
            resource_type: ExtensionResourceType::Text,
            path: "resources/reference.txt".to_string(),
        }],
        ..Default::default()
    }
}

#[test]
fn contributions_accept_r0_skill_and_resource_shapes() {
    super::validation::contributions(&contributions()).unwrap();
}

#[test]
fn contributions_reject_r0_collections_and_unknown_resource_types() {
    let mut oversized_skills = contributions();
    oversized_skills.skills = (0..=MAX_SKILLS_PER_EXTENSION)
        .map(|index| ExtensionSkill {
            id: format!("skill-{index}"),
            name: format!("skill-{index}"),
            description: "Compétence".to_string(),
            path: "SKILL.md".to_string(),
        })
        .collect();
    assert!(super::validation::contributions(&oversized_skills).is_err());

    let mut oversized_resources = contributions();
    oversized_resources.resources = (0..=MAX_RESOURCES_PER_EXTENSION)
        .map(|index| ExtensionResource {
            id: format!("resource-{index}"),
            name: format!("resource-{index}"),
            description: "Ressource".to_string(),
            resource_type: ExtensionResourceType::Text,
            path: "resources/reference.txt".to_string(),
        })
        .collect();
    assert!(super::validation::contributions(&oversized_resources).is_err());

    assert!(serde_json::from_value::<ExtensionResource>(serde_json::json!({
        "id": "unknown",
        "name": "Inconnue",
        "description": "Type inconnu.",
        "type": "archive",
        "path": "resources/archive.bin"
    }))
    .is_err());
}

#[test]
fn resource_paths_use_the_contract_unicode_scalar_limit() {
    let mut exact = contributions();
    exact.resources[0].path = "🦫".repeat(super::types::MAX_PATH_CHARS);
    assert!(super::validation::contributions(&exact).is_ok());

    exact.resources[0].path.push('🦫');
    assert!(super::validation::contributions(&exact).is_err());
}

#[test]
fn contribution_ids_are_ascii_but_visible_names_are_human_metadata() {
    let mut values = contributions();
    values.skills[0].name = "Compétence 🦫".to_string();
    values.resources[0].name = "Référence 🦫".to_string();
    super::validation::contributions(&values).unwrap();

    values.skills[0].id = "compétence".to_string();
    assert!(super::validation::contributions(&values).is_err());
}

#[test]
fn generated_resource_type_serializes_with_its_contract_value() {
    assert_eq!(
        serde_json::to_string(&ExtensionResourceType::Image).unwrap(),
        "\"image\""
    );
    assert!(serde_json::from_str::<ExtensionResourceType>("\"archive\"").is_err());
}
