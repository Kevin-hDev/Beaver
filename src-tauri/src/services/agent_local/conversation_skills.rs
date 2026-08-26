use std::collections::HashSet;

use crate::models::agent_turn_contract::{
    SkillReference, MAX_SKILLS_PER_TURN, MAX_SKILL_NAME_BYTES,
};

use super::conversation_input::{ConversationInputError, ConversationInputErrorKind};

const MAX_REFERENCE_NAME_CHARS: usize = 120;
const MAX_RESOLVED_SKILLS_BYTES: usize =
    MAX_SKILLS_PER_TURN * super::skill_limits::MAX_RESOLVED_SKILL_BYTES;

#[derive(Debug)]
pub struct ResolvedSkill {
    pub id: String,
    pub name: String,
    pub content: String,
}

pub async fn resolve(
    references: Vec<SkillReference>,
) -> Result<Vec<ResolvedSkill>, ConversationInputError> {
    if references.len() > MAX_SKILLS_PER_TURN {
        return Err(error(ConversationInputErrorKind::Limit));
    }
    let mut ids = HashSet::with_capacity(references.len());
    let mut resolved = Vec::with_capacity(references.len());
    let mut total_bytes = 0_usize;
    for reference in references {
        validate_reference(&reference)?;
        if !ids.insert(reference.id.clone()) {
            return Err(error(ConversationInputErrorKind::Invalid));
        }
        let loaded = super::tool_skill_loader::load_skill_with_metadata(&reference.id)
            .await
            .map_err(|_| error(ConversationInputErrorKind::Skill))?;
        if loaded.content.len() > super::skill_limits::MAX_RESOLVED_SKILL_BYTES {
            return Err(error(ConversationInputErrorKind::Limit));
        }
        total_bytes = total_bytes
            .checked_add(loaded.content.len())
            .ok_or_else(|| error(ConversationInputErrorKind::Limit))?;
        if total_bytes > MAX_RESOLVED_SKILLS_BYTES {
            return Err(error(ConversationInputErrorKind::Limit));
        }
        resolved.push(ResolvedSkill {
            id: reference.id,
            name: loaded.name,
            content: loaded.content,
        });
    }
    Ok(resolved)
}

pub(super) fn validate_reference(reference: &SkillReference) -> Result<(), ConversationInputError> {
    if !super::tool_skill_loader::valid_skill_id(&reference.id)
        || reference.name.as_ref().is_some_and(|name| {
            name.is_empty()
                || name.len() > MAX_SKILL_NAME_BYTES
                || name.chars().count() > MAX_REFERENCE_NAME_CHARS
                || name.chars().any(char::is_control)
        })
    {
        return Err(error(ConversationInputErrorKind::Invalid));
    }
    Ok(())
}

pub(super) fn validate_persisted_references(
    ids: Option<&[String]>,
    names: Option<&[String]>,
) -> Result<(), ConversationInputError> {
    let names = names.unwrap_or_default();
    if names.len() > MAX_SKILLS_PER_TURN {
        return Err(error(ConversationInputErrorKind::Invalid));
    }
    let Some(ids) = ids else {
        return Ok(()); // Legacy v2 : aucun ID ni corps n'est déduit du nom visible.
    };
    if names.iter().any(|name| {
            name.is_empty()
                || name.len() > MAX_SKILL_NAME_BYTES
                || name.chars().count() > MAX_REFERENCE_NAME_CHARS
                || name.chars().any(char::is_control)
        })
    {
        return Err(error(ConversationInputErrorKind::Invalid));
    }
    if ids.is_empty() || ids.len() != names.len() || ids.len() > MAX_SKILLS_PER_TURN {
        return Err(error(ConversationInputErrorKind::Invalid));
    }
    let mut unique = HashSet::with_capacity(ids.len());
    for (id, name) in ids.iter().zip(names) {
        let reference = SkillReference {
            id: id.clone(),
            name: Some(name.clone()),
        };
        if !unique.insert(id.as_str()) || validate_reference(&reference).is_err() {
            return Err(error(ConversationInputErrorKind::Invalid));
        }
    }
    Ok(())
}

fn error(kind: ConversationInputErrorKind) -> ConversationInputError {
    ConversationInputError::new(kind)
}
