#![expect(
    clippy::too_many_arguments,
    reason = "interrupted-turn cleanup needs the live loop state"
)]

use super::agent_loop_compression::{LastCounts, LoopCompression};
use super::conversation_journal::ConversationJournal;
use super::stream_events::AgentEventEmitter;
use super::types_ollama::{ChatMessage, StreamResult};
use tokio_util::sync::CancellationToken;

pub(super) async fn handle(
    on_event: &AgentEventEmitter,
    messages: &mut Vec<ChatMessage>,
    result: &StreamResult,
    plan_active: bool,
    input_tokens: u32,
    configured_context: u64,
    compression: &LoopCompression<'_>,
    last_prompt: &mut Option<u32>,
    last_eval: &mut Option<u32>,
    cancel: CancellationToken,
    journal: Option<&mut ConversationJournal>,
) -> Result<(), String> {
    if let Some(journal) = journal {
        journal
            .persist_partial(super::agent_loop_support::build_assistant_message(result))
            .await?;
    }
    super::stream_buffer::finalize_interrupted_content(on_event, result, plan_active);
    super::context_usage_runtime::emit_result(on_event, input_tokens, result, configured_context);
    compression
        .handle_interrupted(
            messages,
            result,
            LastCounts::new(last_prompt, last_eval),
            cancel,
        )
        .await
}
