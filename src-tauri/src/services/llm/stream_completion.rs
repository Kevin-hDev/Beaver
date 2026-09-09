use crate::services::agent_local::{
    context_usage_runtime, conversation_journal::ConversationJournal, stream_buffer,
    stream_events::AgentEventEmitter, types_ollama::StreamResult,
};

pub(super) fn terminal_error(result: &StreamResult) -> Option<&'static str> {
    match result.done_reason.as_deref() {
        Some("length") => Some("provider_output_limit"),
        Some("content_filter") => Some("provider_content_filtered"),
        _ => None,
    }
}

/// Only Chat Completions readers call this. Native clients keep their own
/// completion contracts; a transport [DONE] does not prove a useful answer.
pub(super) fn finish(result: &mut StreamResult) {
    result.completion_error = terminal_error(result).or_else(|| {
        (result.content.trim().is_empty() && result.tool_calls.is_empty())
            .then_some("provider_empty_response")
    });
}

pub(super) async fn reject_if_failed(
    on_event: &AgentEventEmitter,
    result: &StreamResult,
    plan_active: bool,
    journal: Option<&mut ConversationJournal>,
    input_tokens: u32,
    configured_context: u64,
) -> Result<(), String> {
    let Some(error) = result.completion_error else {
        return Ok(());
    };
    if let Some(journal) = journal {
        if !result.content.is_empty()
            || !result.thinking.is_empty()
            || result.continuation.is_some()
        {
            let mut message = super::agent_loop_message::build_assistant_message(result);
            // A truncated tool must never become a runnable or replayable call.
            message.tool_calls = None;
            journal.persist_partial(message).await?;
        }
    }
    stream_buffer::finalize_interrupted_content(on_event, result, plan_active);
    context_usage_runtime::emit_result(on_event, input_tokens, result, configured_context);
    Err(error.to_string())
}

pub(super) fn require_complete(result: StreamResult) -> Result<StreamResult, String> {
    match result.completion_error {
        Some(error) => Err(error.to_string()),
        None => Ok(result),
    }
}

#[cfg(test)]
#[path = "stream_completion_tests.rs"]
mod tests;
