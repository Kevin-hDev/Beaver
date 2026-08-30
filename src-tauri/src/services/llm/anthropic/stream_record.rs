use super::stream_state::StreamState;
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{StreamEvent, StreamResult};

pub(super) fn thinking(
    on_event: &AgentEventEmitter,
    result: &mut StreamResult,
    state: &StreamState,
    start: usize,
    token_count: &mut u32,
) {
    crate::services::agent_local::stream_buffer::record_thinking(
        on_event,
        result,
        state.thinking.chars().skip(start).collect(),
        token_count,
    );
}

pub(super) fn tool(
    on_event: &AgentEventEmitter,
    result: &mut StreamResult,
    state: &StreamState,
    tools: &[serde_json::Value],
    index: usize,
    token_count: &mut u32,
) {
    let (wire_name, arguments) = &state.tool_calls[index];
    let name = crate::services::llm::tool_schema::restore_tool_name(wire_name, tools);
    let id = state.tool_call_ids[index].clone();
    crate::services::agent_local::stream_buffer::record_tool_call_generation(
        on_event,
        result,
        &name,
        arguments,
        token_count,
    );
    let _ = on_event.send(StreamEvent::ToolCall {
        name: name.clone(),
        arguments: arguments.clone(),
        tool_call_index: index,
        tool_call_id: Some(id.clone()),
        domain: crate::services::agent_local::memory_tool::event_domain(&name, arguments),
    });
    result.tool_calls.push((name, arguments.clone()));
    result.tool_call_ids.push(id);
    result.tool_call_extra_content.push(None);
}
