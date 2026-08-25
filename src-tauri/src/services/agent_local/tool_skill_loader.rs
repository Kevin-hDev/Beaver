use crate::services::agent_local::skill_catalog;
use crate::services::agent_local::types_tools::SkillInfo;

pub(crate) const MAX_SKILL_ID_BYTES: usize = crate::models::agent_turn_contract::MAX_SKILL_ID_BYTES;
const MAX_DISPLAY_NAME_CHARS: usize = 120;

pub struct LoadedSkill {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillLoadError {
    InvalidId,
    NotFound,
    Unavailable,
}

impl SkillLoadError {
    pub fn message(self) -> &'static str {
        match self {
            Self::InvalidId => "Identifiant de skill invalide",
            Self::NotFound => "Skill introuvable",
            Self::Unavailable => "Skill indisponible",
        }
    }
}

pub async fn list_skills() -> Result<Vec<SkillInfo>, String> {
    tokio::task::spawn_blocking(|| {
        skill_catalog::entries().map(|entries| {
            entries
                .into_iter()
                .map(|entry| entry.info)
                .collect::<Vec<_>>()
        })
    })
    .await
    .map_err(|_| "Skills indisponibles".to_string())?
}

pub async fn load_skill(skill_id: &str) -> Result<String, String> {
    load_skill_with_metadata(skill_id)
        .await
        .map(|loaded| loaded.content)
        .map_err(|error| error.message().to_string())
}

pub async fn load_skill_with_metadata(skill_id: &str) -> Result<LoadedSkill, SkillLoadError> {
    if !valid_skill_id(skill_id) {
        return Err(SkillLoadError::InvalidId);
    }
    let requested = skill_id.to_string();
    tokio::task::spawn_blocking(move || {
        let entry = skill_catalog::entries()
            .map_err(|_| SkillLoadError::Unavailable)?
            .into_iter()
            .find(|entry| entry.info.id == requested)
            .ok_or(SkillLoadError::NotFound)?;
        let content = super::skill_manifest_read::read(&entry.manifest)
            .map_err(|_| SkillLoadError::Unavailable)?;
        let source = entry.info.source_name;
        let directory = entry
            .bundle_root
            .to_str()
            .ok_or(SkillLoadError::Unavailable)?;
        if source.len() > super::skill_limits::MAX_SKILL_SOURCE_NAME_BYTES
            || directory.len() > super::skill_limits::MAX_SKILL_BUNDLE_PATH_BYTES
            || source.chars().any(char::is_control)
            || directory.chars().any(char::is_control)
        {
            return Err(SkillLoadError::Unavailable);
        }
        let (_, _, body) = crate::services::agent_local::skill_parser::parse_skill_content(
            &content,
            &entry.info.name,
        );
        let enriched = format!(
            "Skill source: {source}\nSkill directory: {directory}\n\n{body}"
        );
        if enriched.len() > super::skill_limits::MAX_RESOLVED_SKILL_BYTES {
            return Err(SkillLoadError::Unavailable);
        }
        Ok(LoadedSkill {
            name: display_name(&entry.info.name),
            content: enriched,
        })
    })
    .await
    .map_err(|_| SkillLoadError::Unavailable)?
}

pub(crate) fn valid_skill_id(skill_id: &str) -> bool {
    !skill_id.is_empty()
        && skill_id.len() <= MAX_SKILL_ID_BYTES
        && !skill_id.contains("..")
        && !skill_id
            .chars()
            .any(|value| matches!(value, '/' | '\\') || value.is_control())
}

fn display_name(name: &str) -> String {
    let bounded = name
        .chars()
        .map(|value| if value.is_control() { ' ' } else { value })
        .take(MAX_DISPLAY_NAME_CHARS)
        .collect::<String>();
    let trimmed = bounded.trim();
    if trimmed.is_empty() {
        "Skill".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{display_name, MAX_DISPLAY_NAME_CHARS};

    #[test]
    fn display_name_is_single_line_and_bounded() {
        let name = format!("context7\n{}", "x".repeat(200));
        let display = display_name(&name);

        assert!(!display.contains('\n'));
        assert_eq!(display.chars().count(), MAX_DISPLAY_NAME_CHARS);
    }

    #[test]
    fn display_name_falls_back_when_empty() {
        assert_eq!(display_name("\n\t"), "Skill");
    }
}
