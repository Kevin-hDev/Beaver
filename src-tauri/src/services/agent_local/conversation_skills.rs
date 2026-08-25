use std::collections::HashSet;

use crate::models::agent_turn_contract::{
    SkillReference, MAX_SKILLS_PER_TURN, MAX_SKILL_NAME_BYTES,
};

use super::conversation_input::{ConversationInputError, ConversationInputErrorKind};

const MAX_REFERENCE_NAME_CHARS: usize = 120;
const MAX_RESOLVED_SKILL_BYTES: usize = 256 * 1024;
const MAX_RESOLVED_SKILLS_BYTES: usize = MAX_SKILLS_PER_TURN * MAX_RESOLVED_SKILL_BYTES;

#[allow(dead_code, reason = "consumed by conversation admission in Task 8")]
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
        if loaded.content.len() > MAX_RESOLVED_SKILL_BYTES {
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

fn validate_reference(reference: &SkillReference) -> Result<(), ConversationInputError> {
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

fn error(kind: ConversationInputErrorKind) -> ConversationInputError {
    ConversationInputError::new(kind)
}
