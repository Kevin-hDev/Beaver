use super::stream_events::AgentEventEmitter;
use super::types_stream::{StreamEvent, StreamResult};
use crate::services::token_counting;

pub fn emit_input(
    on_event: &AgentEventEmitter,
    input_tokens: usize,
    context_limit: u64,
) -> u32 {
    let input_tokens = bounded_tokens(input_tokens);
    emit(on_event, input_tokens, 0, context_limit, true);
    input_tokens
}

pub fn emit_result(
    on_event: &AgentEventEmitter,
    estimated_input_tokens: u32,
    result: &StreamResult,
    context_limit: u64,
) {
    let (input, output, estimated) = resolved_usage(estimated_input_tokens, result);
    emit(on_event, input, output, context_limit, estimated);
}

impl StreamResult {
    pub fn record_generated_text(&mut self, text: &str) -> u32 {
        self.generated_units = self
            .generated_units
            .saturating_add(token_counting::text_units(text));
        self.estimated_output_tokens()
    }

    pub fn record_generated_tool_call(&mut self, name: &str, arguments: &serde_json::Value) {
        self.generated_units = self
            .generated_units
            .saturating_add(token_counting::text_units(name))
            .saturating_add(token_counting::text_units(&arguments.to_string()));
    }

    pub fn estimated_output_tokens(&self) -> u32 {
        bounded_tokens(token_counting::token_count_from_units(
            self.generated_units,
        ))
    }
}

fn resolved_usage(estimated_input_tokens: u32, result: &StreamResult) -> (u32, u32, bool) {
    let input = result.prompt_tokens.unwrap_or(estimated_input_tokens);
    let output = result
        .eval_count
        .unwrap_or_else(|| result.estimated_output_tokens());
    let estimated = result.prompt_tokens.is_none() || result.eval_count.is_none();
    (input, output, estimated)
}

fn emit(
    on_event: &AgentEventEmitter,
    input_tokens: u32,
    output_tokens: u32,
    context_limit: u64,
    estimated: bool,
) {
    let _ = on_event.send(StreamEvent::ContextUsage {
        input_tokens,
        output_tokens,
        context_tokens: input_tokens.saturating_add(output_tokens),
        context_limit: context_limit.min(u32::MAX as u64) as u32,
        estimated,
    });
}

fn bounded_tokens(tokens: usize) -> u32 {
    tokens.min(u32::MAX as usize) as u32
}

#[cfg(test)]
#[path = "context_usage_runtime_tests.rs"]
mod tests;
