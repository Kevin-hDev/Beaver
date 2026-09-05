use super::tool_skill_loader::{LoadedSkill, SkillLoadError};

pub(crate) async fn load_skill_for_session(
    skill_id: &str,
    session_id: &str,
) -> Result<LoadedSkill, SkillLoadError> {
    if !skill_id.starts_with("extension:") {
        return super::tool_skill_loader::load_skill_with_metadata(skill_id).await;
    }
    let loaded =
        crate::services::extensions::load_extension_skill_for_session(skill_id, session_id)
            .await
            .map_err(map_error)?;
    enrich(loaded.name, loaded.extension_id, loaded.bytes)
}

pub(super) fn enrich(
    name: String,
    extension_id: String,
    bytes: Vec<u8>,
) -> Result<LoadedSkill, SkillLoadError> {
    let content = String::from_utf8(bytes).map_err(|_| SkillLoadError::Unavailable)?;
    let (_, _, body) = super::skill_parser::parse_skill_content(&content, &name);
    let enriched = format!("Skill source: {extension_id}\n\n{body}");
    if enriched.len() > super::skill_limits::MAX_RESOLVED_SKILL_BYTES {
        return Err(SkillLoadError::Unavailable);
    }
    Ok(LoadedSkill {
        name: super::tool_skill_loader::display_name(&name),
        content: enriched,
    })
}

pub(super) fn map_error(error: crate::services::extensions::ResourceLoadError) -> SkillLoadError {
    match error {
        crate::services::extensions::ResourceLoadError::InvalidId => SkillLoadError::InvalidId,
        crate::services::extensions::ResourceLoadError::TooLarge => SkillLoadError::Unavailable,
        crate::services::extensions::ResourceLoadError::NotFound => SkillLoadError::NotFound,
        crate::services::extensions::ResourceLoadError::Unavailable => SkillLoadError::Unavailable,
    }
}
