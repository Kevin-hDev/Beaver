use super::types_ollama::StreamEvent;
use super::generation_metrics::GenerationAggregate;
use crate::services::token_counting;

pub fn done_event(
    total_eval: Option<u32>,
    total_prompt: Option<u32>,
    last_prompt: Option<u32>,
    last_eval: Option<u32>,
    generation: GenerationAggregate,
) -> (StreamEvent, u32) {
    let summary = generation.summary();
    let event = StreamEvent::Done {
        eval_count: total_eval,
        eval_duration_ns: summary.duration_ns,
        final_tps: summary.tps,
        tps_estimated: summary.estimated,
        prompt_tokens: total_prompt,
        context_tokens: token_counting::sum_real_counts(last_prompt, last_eval),
    };
    (event, token_counting::sum_real_counts(total_eval, total_prompt).unwrap_or(0))
}
