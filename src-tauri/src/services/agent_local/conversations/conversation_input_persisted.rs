use std::collections::HashSet;

use crate::models::agent_turn_contract::{NewUserTurnInput, TurnAttachmentInput};

use super::conversation_attachments::ResolvedAttachmentContent;
use super::conversation_input::{
    ConversationInputError, ResolvedTurnInput, MAX_TEXT_CHARS_PER_TURN,
};
use super::types_message::FileAttachment;

pub async fn resolve(
    input: NewUserTurnInput,
    key: &[u8],
) -> Result<ResolvedTurnInput, ConversationInputError> {
    super::conversation_input::validate_top_level(&input)?;
    let mut provider_content = input.content.clone();
    let mut files = Vec::with_capacity(input.files.len());
    let mut images = Vec::new();
    let mut text_chars = 0usize;
    let mut attachment_keys = HashSet::with_capacity(input.files.len());

    for file in input.files {
        let fallback = persisted_file(&file);
        match super::conversation_attachments::resolve(file, key).await {
            Ok(resolved) if attachment_keys.insert(resolved.identity()) => {
                match resolved.content {
                    ResolvedAttachmentContent::Text(text)
                        if text_chars.saturating_add(text.chars().count())
                            <= MAX_TEXT_CHARS_PER_TURN =>
                    {
                        text_chars = text_chars.saturating_add(text.chars().count());
                        append_text(&mut provider_content, &resolved.file.name, &text);
                    }
                    ResolvedAttachmentContent::Image(image)
                        if images.len() < crate::services::llm::vision::MAX_IMAGES_PER_MESSAGE =>
                    {
                        images.push(image);
                    }
                    _ => append_unavailable(&mut provider_content, &fallback.name),
                }
                files.push(resolved.file);
            }
            _ => {
                append_unavailable(&mut provider_content, &fallback.name);
                files.push(fallback);
            }
        }
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

fn persisted_file(input: &TurnAttachmentInput) -> FileAttachment {
    FileAttachment {
        name: input.name.clone(),
        path: input.path.clone(),
        mime_type: input.mime_type.clone(),
        size: input.size,
        thumbnail: input.thumbnail.clone(),
        access_grant: input.access_grant.clone(),
    }
}

fn append_text(content: &mut String, name: &str, text: &str) {
    content.push_str("\n\n--- File: ");
    content.push_str(name);
    content.push_str(" ---\n");
    content.push_str(text);
}

fn append_unavailable(content: &mut String, name: &str) {
    content.push_str("\n\n--- Attachment: ");
    content.push_str(name);
    content.push_str(" ---\n[attachment unavailable]");
}
