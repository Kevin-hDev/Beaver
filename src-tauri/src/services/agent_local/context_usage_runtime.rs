use super::stream_events::AgentEventEmitter;
use super::types_stream::{StreamEvent, StreamResult};
use crate::services::token_counting;

pub fn prepared_input_tokens(
    provider_id: &str,
    estimated_tokens: usize,
    messages: &[super::types_ollama::ChatMessage],
    tools: &[serde_json::Value],
) -> usize {
    if provider_id != "codex-oauth" {
        return estimated_tokens;
    }
    token_counting::estimate_chat_tokens_without_reasoning(messages)
        .saturating_add(crate::services::compress::token_estimate::estimate_tool_tokens(tools))
}

pub fn emit_input(on_event: &AgentEventEmitter, input_tokens: usize) -> u32 {
    let input_tokens = bounded_tokens(input_tokens);
    emit(on_event, input_tokens, 0, true);
    input_tokens
}

pub fn emit_result(
    on_event: &AgentEventEmitter,
    estimated_input_tokens: u32,
    result: &StreamResult,
) {
    match (result.prompt_tokens, result.eval_count) {
        (Some(input), Some(output)) => emit(on_event, input, output, false),
        _ => emit(
            on_event,
            estimated_input_tokens,
            result.estimated_output_tokens(),
            true,
        ),
    }
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

fn emit(on_event: &AgentEventEmitter, input_tokens: u32, output_tokens: u32, estimated: bool) {
    let _ = on_event.send(StreamEvent::ContextUsage {
        input_tokens,
        output_tokens,
        context_tokens: input_tokens.saturating_add(output_tokens),
        estimated,
    });
}

fn bounded_tokens(tokens: usize) -> u32 {
    tokens.min(u32::MAX as usize) as u32
}

#[cfg(test)]
#[path = "context_usage_runtime_tests.rs"]
mod tests;
