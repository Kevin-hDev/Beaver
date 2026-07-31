use super::stream_events::AgentEventEmitter;
use super::types_stream::StreamResult;
use crate::services::stream_utils::{FilteredChunk, ThinkTagFilter};

pub(super) fn emit_filtered(
    filter: &mut ThinkTagFilter,
    content: &str,
    on_event: &AgentEventEmitter,
    token_count: &mut u32,
    result: &mut StreamResult,
    buffer_content: bool,
) {
    record_chunks(
        filter.feed(content),
        on_event,
        token_count,
        result,
        buffer_content,
    );
}

pub(crate) fn flush_filter(
    filter: &mut ThinkTagFilter,
    on_event: &AgentEventEmitter,
    token_count: &mut u32,
    result: &mut StreamResult,
    buffer_content: bool,
) {
    record_chunks(
        filter.flush(),
        on_event,
        token_count,
        result,
        buffer_content,
    );
}

fn record_chunks(
    chunks: Vec<FilteredChunk>,
    on_event: &AgentEventEmitter,
    token_count: &mut u32,
    result: &mut StreamResult,
    buffer_content: bool,
) {
    for chunk in chunks {
        match chunk {
            FilteredChunk::Thinking(content) => super::stream_buffer::record_thinking(
                on_event,
                result,
                content,
                token_count,
            ),
            FilteredChunk::Content(content) => super::stream_buffer::record_content(
                on_event,
                result,
                content,
                token_count,
                buffer_content,
            ),
        }
    }
}
