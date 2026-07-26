use super::agent_loop_completion;
use super::agent_loop_support;
use super::stream_diagnostics;
use super::stream_events::AgentEventEmitter;
use super::types_ollama::StreamEvent;
use std::time::Instant;

pub type CompletionCounts = (Option<u32>, Option<u32>, Option<u32>, Option<u32>);

pub fn emit_turn_end(on_event: &AgentEventEmitter, compressed_after_tools: bool) {
    if !compressed_after_tools {
        let _ = on_event.send(StreamEvent::TurnEnd {});
    }
}

pub async fn finish(
    on_event: &AgentEventEmitter,
    counts: CompletionCounts,
    start: Instant,
    request: (&str, &str),
    ollama_model: Option<&str>,
) -> u32 {
    let token_total = agent_loop_completion::emit_done(
        on_event, counts.0, counts.1, counts.2, counts.3, start,
    );
    stream_diagnostics::record_completed(request.0, request.1).await;
    if let Some(model) = ollama_model {
        agent_loop_support::decharge_gpu(model).await;
    }
    token_total
}
