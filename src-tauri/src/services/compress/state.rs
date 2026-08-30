#[cfg(test)]
use crate::services::agent_local::session_store;
#[cfg(test)]
use crate::services::agent_local::types_ollama::ChatMessage;
#[cfg(test)]
use crate::services::compress::{context_capsules_disk, token_estimate};
#[cfg(test)]
use std::path::Path;

#[cfg(test)]
pub use context_capsules_disk::CompressionMode;

pub fn context_used_for_compression(
    last_context_tokens: Option<u32>,
    estimated_tokens: usize,
) -> usize {
    last_context_tokens
        .map(|tokens| std::cmp::max(tokens as usize, estimated_tokens))
        .unwrap_or(estimated_tokens)
}

#[cfg(test)]
pub fn is_safe_to_compress(messages: &[ChatMessage]) -> bool {
    super::state_recent::tool_chain_is_closed(messages)
}

#[cfg(test)]
pub async fn apply_and_save(
    session_id: &str,
    runtime_messages: &mut Vec<ChatMessage>,
    summary: &str,
    context_window: u64,
    suppress_follow_up: bool,
    working_dir: &Path,
    mode: CompressionMode,
) -> Result<u32, String> {
    let session = session_store::get(session_id).await?;
    let context = context_capsules_disk::compression_context_message(
        runtime_messages,
        context_window,
        working_dir,
        mode,
    )
    .await;
    let profile = super::profile_resolve::resolve_for_session(&session)
        .map_err(|_| "Compression impossible".to_string())?;
    let capabilities = super::session_capabilities::SessionCompressionCapabilities::from_runtime(
        false,
        &[],
        false,
        false,
        false,
    )?;
    let trigger = match mode {
        CompressionMode::Manual => super::profile_types::CompressionTrigger::Explicit,
        CompressionMode::Auto { .. } => super::profile_types::CompressionTrigger::Automatic,
    };
    let canonical = runtime_messages
        .iter()
        .filter(|message| message.role == "system")
        .cloned()
        .collect();
    let before_tokens =
        token_estimate::estimate_tokens(runtime_messages).min(u32::MAX as usize) as u32;
    let snapshot = super::snapshot::CompressionSnapshot::capture(
        &session,
        profile,
        context_window,
        capabilities,
        trigger,
    )?
    .with_runtime_context(canonical, Vec::new(), before_tokens)?;
    let validated = super::summary_contract::ValidatedSummary {
        content: if suppress_follow_up {
            summary.to_string()
        } else {
            format!("{summary}\n\nContinue from this checkpoint.")
        },
        estimated_tokens: crate::services::token_counting::estimate_text_tokens(summary)
            .min(u32::MAX as usize) as u32,
    };
    let sections = context
        .map(|message| {
            vec![super::checkpoint_document::CheckpointSection {
                name: "recent_file_context".to_string(),
                content: message.content,
            }]
        })
        .unwrap_or_default();
    let candidate = super::checkpoint_candidate::build(&snapshot, Some(&validated), &sections)
        .await
        .map_err(|error| error.public_message().to_string())?;
    super::checkpoint_transaction::commit_candidate(session_id, runtime_messages, candidate)
        .await
        .map(|report| report.after_tokens)
        .map_err(|error| error.public_message().to_string())
}

#[cfg(test)]
pub fn request_start_index(messages: &[ChatMessage]) -> usize {
    let segment_start = messages
        .iter()
        .rposition(|message| message.continuity_barrier_before)
        .unwrap_or(0);
    messages[segment_start..]
        .iter()
        .rposition(|message| {
            message.role == "user" && super::state_recent::include_chat_message(message)
        })
        .map(|offset| segment_start + offset)
        .unwrap_or(messages.len())
}
