use super::stream_events::AgentEventEmitter;
use super::types_ollama::StreamEvent;
use crate::services::token_counting;
use std::time::Instant;

pub fn emit_done(
    on_event: &AgentEventEmitter,
    total_eval: Option<u32>,
    total_prompt: Option<u32>,
    last_prompt: Option<u32>,
    last_eval: Option<u32>,
    start: Instant,
) -> u32 {
    let elapsed_ns = start.elapsed().as_nanos() as u64;
    let final_tps = if elapsed_ns > 0 {
        total_eval.unwrap_or(0) as f64 / (elapsed_ns as f64 / 1e9)
    } else {
        0.0
    };
    let _ = on_event.send(StreamEvent::Done {
        eval_count: total_eval,
        eval_duration_ns: elapsed_ns,
        final_tps,
        prompt_tokens: total_prompt,
        context_tokens: token_counting::sum_real_counts(last_prompt, last_eval),
    });
    token_counting::sum_real_counts(total_eval, total_prompt).unwrap_or(0)
}
