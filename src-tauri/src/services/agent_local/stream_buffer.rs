use super::stream_events::AgentEventEmitter;
use super::types_stream::{StreamEvent, StreamResult, TokenPhase};
use crate::services::stream_utils::compute_tps;

pub fn record_content(
    on_event: &AgentEventEmitter,
    result: &mut StreamResult,
    content: String,
    token_count: &mut u32,
    first_token: &mut Option<std::time::Instant>,
    buffer_content: bool,
) {
    result.content.push_str(&content);
    result.content_chunks.push(content.clone());
    *token_count = result.record_generated_text(&content);
    if first_token.is_none() {
        *first_token = Some(std::time::Instant::now());
    }
    if !buffer_content {
        emit_token(on_event, content, *token_count, *first_token, None);
    }
}

pub fn record_thinking(
    on_event: &AgentEventEmitter,
    result: &mut StreamResult,
    content: String,
    token_count: &mut u32,
    first_token: &mut Option<std::time::Instant>,
) {
    result.thinking.push_str(&content);
    *token_count = result.record_generated_text(&content);
    if first_token.is_none() {
        *first_token = Some(std::time::Instant::now());
    }
    let _ = on_event.send(StreamEvent::Thinking {
        content,
        token_count: *token_count,
    });
}

pub fn emit_buffered_content(
    on_event: &AgentEventEmitter,
    result: &StreamResult,
    phase: TokenPhase,
) {
    let mut units = crate::services::token_counting::text_units(&result.thinking);
    let first_token = Some(std::time::Instant::now());
    for chunk in &result.content_chunks {
        units = units.saturating_add(crate::services::token_counting::text_units(chunk));
        let token_count = crate::services::token_counting::token_count_from_units(units)
            .min(u32::MAX as usize) as u32;
        emit_token(
            on_event,
            chunk.clone(),
            token_count,
            first_token,
            Some(phase.clone()),
        );
    }
}

pub fn finalize_content_phase(
    on_event: &AgentEventEmitter,
    result: &StreamResult,
    plan_active: bool,
    force_work_phase: bool,
) {
    if let Some(phase) = content_phase_for_result(result, plan_active, force_work_phase) {
        if plan_active {
            emit_buffered_content(on_event, result, phase);
        } else {
            let _ = on_event.send(StreamEvent::ContentPhase { phase });
        }
    }
}

pub fn finalize_interrupted_content(
    on_event: &AgentEventEmitter,
    result: &StreamResult,
    plan_active: bool,
) {
    if interrupted_phase_for_result(result).is_none() {
        return;
    }
    if plan_active {
        emit_buffered_content(on_event, result, TokenPhase::Work);
    } else {
        let _ = on_event.send(StreamEvent::ContentPhase {
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
    on_event: &AgentEventEmitter,
    content: String,
    token_count: u32,
    first_token: Option<std::time::Instant>,
    phase: Option<TokenPhase>,
) {
    let tps = compute_tps(token_count, first_token);
    let _ = on_event.send(StreamEvent::Token {
        content,
        token_count,
        tps,
        phase,
    });
}

#[cfg(test)]
#[path = "stream_buffer_tests.rs"]
mod tests;
