use serde::Serialize;

use super::profile_types::ItemBudget;

#[derive(Serialize)]
pub struct TextAttachmentCheckpoint {
    message_id: String,
    name: String,
    content: String,
}

pub async fn collect(
    messages: &[crate::services::agent_local::types_message::AgentMessage],
    budget: &ItemBudget,
) -> Vec<TextAttachmentCheckpoint> {
    use crate::models::agent_turn_contract::{NewUserTurnInput, TurnAttachmentInput};

    let key = crate::services::attachment_access::attachment_key().ok();
    let key = key.as_ref().map_or(&[][..], |value| value.as_slice());
    let mut output = Vec::new();
    let mut remaining = if budget.total_tokens == 0 {
        u32::MAX
    } else {
        budget.total_tokens
    };
    for message in messages
        .iter()
        .rev()
        .filter(|message| message.role == "user")
    {
        for file in message.files.iter().rev() {
            if output.len() >= usize::from(budget.max_items) || file.mime_type.starts_with("image/")
            {
                continue;
            }
            let input = NewUserTurnInput {
                content: String::new(),
                files: vec![TurnAttachmentInput {
                    name: file.name.clone(),
                    path: file.path.clone(),
                    mime_type: file.mime_type.clone(),
                    size: file.size,
                    thumbnail: file.thumbnail.clone(),
                    access_grant: file.access_grant.clone(),
                }],
                skills: Vec::new(),
            };
            let resolved =
                crate::services::agent_local::conversation_input::resolve_persisted_with_key(
                    input, key,
                )
                .await
                .map(|value| value.provider_content)
                .unwrap_or_else(|_| "[attachment unavailable]".to_string());
            let content = super::checkpoint_messages::bounded_excerpt(
                &resolved,
                budget.tokens_per_item,
                "\n[attachment truncated]",
                "",
            );
            let tokens = crate::services::token_counting::estimate_text_tokens(&content)
                .min(u32::MAX as usize) as u32;
            if tokens > remaining {
                continue;
            }
            remaining = remaining.saturating_sub(tokens);
            output.push(TextAttachmentCheckpoint {
                message_id: message.id.clone(),
                name: file.name.clone(),
                content,
            });
        }
    }
    output.reverse();
    output
}
