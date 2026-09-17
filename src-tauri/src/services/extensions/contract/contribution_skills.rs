use std::collections::HashSet;

use super::types::ExtensionSkill;

pub fn validate(values: &[ExtensionSkill]) -> Result<(), ()> {
    if values.len() > super::types::MAX_SKILLS_PER_EXTENSION {
        return Err(());
    }
    let mut ids = HashSet::with_capacity(values.len());
    for value in values {
        if !ids.insert(value.id.as_str())
            || super::validation::identifier(&value.id).is_err()
            || super::validation::contribution_text(
                &value.name,
                super::types::MAX_EXTENSION_NAME_CHARS,
            )
            .is_err()
            || super::validation::contribution_text(
                &value.description,
                super::types::MAX_EXTENSION_TEXT_CHARS,
            )
            .is_err()
            || super::contribution_path::validate(&value.path).is_err()
            || !matches!(value.path.rsplit('/').next(), Some("SKILL.md" | "skill.md"))
        {
            return Err(());
        }
    }
    Ok(())
}
