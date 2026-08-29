use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::StreamResult;
use crate::services::stream_utils::FilteredChunk;

pub(super) fn record_filtered(
    chunk: FilteredChunk,
    on_event: &AgentEventEmitter,
    result: &mut StreamResult,
    token_count: &mut u32,
    buffer_content: bool,
) {
    match chunk {
        FilteredChunk::Thinking(content) => {
            crate::services::agent_local::stream_buffer::record_thinking(
                on_event,
                result,
                content,
                token_count,
            );
        }
        FilteredChunk::Content(content) => {
            crate::services::agent_local::stream_buffer::record_content(
                on_event,
                result,
                content,
                token_count,
                buffer_content,
            );
        }
    }
}
