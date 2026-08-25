use std::collections::HashSet;
use std::fmt;

use crate::models::agent_turn_contract::{
    NewUserTurnInput, MAX_SKILLS_PER_TURN, MAX_TURN_ATTACHMENTS, MAX_TURN_CONTENT_BYTES,
};

use super::conversation_attachments::ResolvedAttachmentContent;

pub const MAX_TEXT_CHARS_PER_FILE: usize = 120_000;
pub const MAX_TEXT_CHARS_PER_TURN: usize = 300_000;
pub const PUBLIC_ERROR_CODE: &str = "conversation_input_invalid";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationInputErrorKind {
    Invalid,
    Grant,
    Type,
    Limit,
    Skill,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConversationInputError {
    kind: ConversationInputErrorKind,
}

impl ConversationInputError {
    pub(crate) fn new(kind: ConversationInputErrorKind) -> Self {
        Self { kind }
    }

    #[allow(dead_code, reason = "consumed by diagnostics from Task 9")]
    pub fn kind(self) -> ConversationInputErrorKind {
        self.kind
    }

    #[allow(dead_code, reason = "consumed by the IPC error adapter from Task 9")]
    pub fn public_code(self) -> &'static str {
        PUBLIC_ERROR_CODE
    }
}

impl fmt::Display for ConversationInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(PUBLIC_ERROR_CODE)
    }
}

impl std::error::Error for ConversationInputError {}

#[allow(dead_code, reason = "adopted by conversation admission in Task 8")]
#[derive(Debug)]
pub struct ResolvedTurnInput {
    pub user_content: String,
    pub provider_content: String,
    pub files: Vec<super::types_message::FileAttachment>,
    pub images: Vec<super::conversation_attachments::ResolvedImage>,
    pub skills: Vec<super::conversation_skills::ResolvedSkill>,
}

#[allow(dead_code, reason = "adopted by the chat IPC boundary in Task 9")]
pub async fn resolve(input: NewUserTurnInput) -> Result<ResolvedTurnInput, ConversationInputError> {
    resolve_with_key_source(input, crate::services::attachment_access::attachment_key).await
}

pub(crate) async fn resolve_with_key(
    input: NewUserTurnInput,
    key: &[u8],
) -> Result<ResolvedTurnInput, ConversationInputError> {
    resolve_with_key_source(input, || Ok(zeroize::Zeroizing::new(key.to_vec()))).await
}

pub(crate) async fn resolve_with_key_source<F>(
    input: NewUserTurnInput,
    key_source: F,
) -> Result<ResolvedTurnInput, ConversationInputError>
where
    F: FnOnce() -> Result<zeroize::Zeroizing<Vec<u8>>, String>,
{
    validate_top_level(&input)?;
    let needs_key = input.files.iter().any(|file| !file.path.is_empty());
    let key = needs_key
        .then(key_source)
        .transpose()
        .map_err(|_| ConversationInputError::new(ConversationInputErrorKind::Unavailable))?;
    let key_bytes: &[u8] = key.as_ref().map_or(&[], |value| value.as_slice());
    let mut provider_content = input.content.clone();
    let mut files = Vec::with_capacity(input.files.len());
    let mut images = Vec::new();
    let mut text_chars = 0_usize;
    let mut attachment_keys = HashSet::with_capacity(input.files.len());

    for file in input.files {
        let resolved = super::conversation_attachments::resolve(file, key_bytes).await?;
        if !attachment_keys.insert(resolved.identity()) {
            return Err(ConversationInputError::new(
                ConversationInputErrorKind::Invalid,
            ));
        }
        match resolved.content {
            ResolvedAttachmentContent::Text(text) => {
                text_chars = text_chars
                    .checked_add(text.chars().count())
                    .ok_or_else(|| {
                        ConversationInputError::new(ConversationInputErrorKind::Limit)
                    })?;
                if text_chars > MAX_TEXT_CHARS_PER_TURN {
                    return Err(ConversationInputError::new(
                        ConversationInputErrorKind::Limit,
                    ));
                }
                provider_content.push_str("\n\n--- File: ");
                provider_content.push_str(&resolved.file.name);
                provider_content.push_str(" ---\n");
                provider_content.push_str(&text);
            }
            ResolvedAttachmentContent::Image(image) => images.push(image),
        }
        files.push(resolved.file);
    }
    if images.len() > crate::services::llm::vision::MAX_IMAGES_PER_MESSAGE {
        return Err(ConversationInputError::new(
            ConversationInputErrorKind::Limit,
        ));
    }
    let skills = super::conversation_skills::resolve(input.skills).await?;
    Ok(ResolvedTurnInput {
        user_content: input.content,
        provider_content,
        files,
        images,
        skills,
    })
}

fn validate_top_level(input: &NewUserTurnInput) -> Result<(), ConversationInputError> {
    if input.content.len() > MAX_TURN_CONTENT_BYTES
        || input.content.contains('\0')
        || input.files.len() > MAX_TURN_ATTACHMENTS
        || input.skills.len() > MAX_SKILLS_PER_TURN
    {
        return Err(ConversationInputError::new(
            ConversationInputErrorKind::Limit,
        ));
    }
    Ok(())
}
