use super::types_stream::{StreamOutcome, StreamResult};

pub(super) fn into_outcome(result: StreamResult, interrupted: bool) -> StreamOutcome {
    eprintln!(
        "[ollama-stream] fin={} content_chars={} thinking_chars={} tool_calls={} done_reason={} chunks={} empty_chunks={}",
        if interrupted { "interrupted" } else { "eof" },
        result.content.chars().count(),
        result.thinking.chars().count(),
        result.tool_calls.len(),
        result.done_reason.as_deref().unwrap_or("none"),
        result.total_chunks,
        result.empty_chunks
    );
    if interrupted {
        StreamOutcome::InterruptedForCompression(result)
    } else {
        StreamOutcome::Completed(result)
    }
}
