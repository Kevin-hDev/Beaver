use super::stream_events::AgentEventEmitter;
use super::types_stream::{StreamEvent, StreamResult, TokenPhase};

pub trait StreamEventSink {
    fn send_event(&self, event: StreamEvent) -> Result<(), String>;
}

impl StreamEventSink for AgentEventEmitter {
    fn send_event(&self, event: StreamEvent) -> Result<(), String> {
        self.send(event)
    }
}

pub fn record_content(
    on_event: &impl StreamEventSink,
    result: &mut StreamResult,
    content: String,
    token_count: &mut u32,
    buffer_content: bool,
) {
    result.content.push_str(&content);
    result.content_chunks.push(content.clone());
    *token_count = result.record_generated_text(&content);
    record_counted_activity(on_event, result, *token_count);
    if !buffer_content {
        emit_token(
            on_event,
            content,
            *token_count,
            result.generation.live_tps(*token_count),
            None,
        );
    }
}

pub fn record_thinking(
    on_event: &impl StreamEventSink,
    result: &mut StreamResult,
    content: String,
    token_count: &mut u32,
) {
    result.thinking.push_str(&content);
    *token_count = result.record_generated_text(&content);
    record_counted_activity(on_event, result, *token_count);
    let _ = on_event.send_event(StreamEvent::Thinking {
        content,
        token_count: *token_count,
    });
}

pub fn record_generation_started(
    on_event: &impl StreamEventSink,
    result: &mut StreamResult,
) {
    if result.generation.start_activity() {
        let _ = on_event.send_event(StreamEvent::GenerationStarted {});
    }
}

pub fn record_tool_call_generation(
    on_event: &impl StreamEventSink,
    result: &mut StreamResult,
    name: &str,
    arguments: &serde_json::Value,
    token_count: &mut u32,
) {
    result.record_generated_tool_call(name, arguments);
    *token_count = result.estimated_output_tokens();
    record_counted_activity(on_event, result, *token_count);
}

pub fn emit_buffered_content(
    on_event: &impl StreamEventSink,
    result: &StreamResult,
    phase: TokenPhase,
) {
    let mut units = crate::services::token_counting::text_units(&result.thinking);
    let mut aggregate = super::generation_metrics::GenerationAggregate::default();
    aggregate.add_result(result);
    let tps = aggregate.summary().tps;
    for chunk in &result.content_chunks {
        units = units.saturating_add(crate::services::token_counting::text_units(chunk));
        let token_count = crate::services::token_counting::token_count_from_units(units)
            .min(u32::MAX as usize) as u32;
        emit_token(on_event, chunk.clone(), token_count, tps, Some(phase.clone()));
    }
}

pub fn finalize_content_phase(
    on_event: &impl StreamEventSink,
    result: &StreamResult,
    plan_active: bool,
    force_work_phase: bool,
) {
    if let Some(phase) = content_phase_for_result(result, plan_active, force_work_phase) {
        if plan_active {
            emit_buffered_content(on_event, result, phase);
        } else {
            let _ = on_event.send_event(StreamEvent::ContentPhase { phase });
        }
    }
}

pub fn finalize_interrupted_content(
    on_event: &impl StreamEventSink,
    result: &StreamResult,
    plan_active: bool,
) {
    if interrupted_phase_for_result(result).is_none() {
        return;
    }
    if plan_active {
        emit_buffered_content(on_event, result, TokenPhase::Work);
    } else {
        let _ = on_event.send_event(StreamEvent::ContentPhase {
            phase: TokenPhase::Work,
        });
    }
}

pub fn content_phase_for_result(
    result: &StreamResult,
    plan_active: bool,
    force_work_phase: bool,
) -> Option<TokenPhase> {
    if result.content_chunks.is_empty() {
        return None;
    }
    if plan_active && !result.tool_calls.is_empty() {
        return None;
    }
    if force_work_phase {
        return Some(TokenPhase::Work);
    }
    Some(if result.tool_calls.is_empty() {
        TokenPhase::Final
    } else {
        TokenPhase::Work
    })
}

pub fn interrupted_phase_for_result(result: &StreamResult) -> Option<TokenPhase> {
    if result.content_chunks.is_empty() {
        return None;
    }
    Some(TokenPhase::Work)
}

fn emit_token(
    on_event: &impl StreamEventSink,
    content: String,
    token_count: u32,
    tps: f64,
    phase: Option<TokenPhase>,
) {
    let _ = on_event.send_event(StreamEvent::Token {
        content,
        token_count,
        tps,
        phase,
    });
}

fn record_counted_activity(
    on_event: &impl StreamEventSink,
    result: &mut StreamResult,
    token_count: u32,
) {
    if result.generation.record_activity(token_count) {
        let _ = on_event.send_event(StreamEvent::GenerationStarted {});
    }
}

#[cfg(test)]
#[path = "stream_buffer_tests.rs"]
mod tests;
