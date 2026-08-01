use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::agent_local::types_ollama::{StreamEvent, StreamResult};

const MAX_TOOL_ARGUMENT_BYTES: usize = 4 * 1024 * 1024;
const MAX_TOOL_ID_BYTES: usize = 512;
const MAX_TOOL_NAME_BYTES: usize = 256;
const MAX_TOOL_CALLS: usize = 32;

pub(super) struct StreamTool<'a> {
    id: Option<String>,
    name: Option<String>,
    arguments: String,
    definitions: &'a [serde_json::Value],
}

impl<'a> StreamTool<'a> {
    pub(super) fn new(definitions: &'a [serde_json::Value]) -> Self {
        Self {
            id: None,
            name: None,
            arguments: String::new(),
            definitions,
        }
    }

    pub(super) fn is_pending(&self) -> bool {
        self.id.is_some()
    }

    pub(super) fn start(
        &mut self,
        on_event: &AgentEventEmitter,
        result: &mut StreamResult,
        event: &serde_json::Value,
    ) -> Result<(), String> {
        let item = event.get("item").ok_or_else(invalid)?;
        if item["type"].as_str() != Some("function_call") {
            return Ok(());
        }
        if self.is_pending() || !has_tool_capacity(result) {
            return Err(invalid());
        }
        let id = bounded(item["call_id"].as_str(), MAX_TOOL_ID_BYTES)?;
        let wire_name = bounded(item["name"].as_str(), MAX_TOOL_NAME_BYTES)?;
        let restored_name =
            crate::services::llm::tool_schema::restore_tool_name(&wire_name, self.definitions);
        let name = bounded(Some(&restored_name), MAX_TOOL_NAME_BYTES)?;
        crate::services::agent_local::stream_buffer::record_generation_started(on_event, result);
        self.id = Some(id);
        self.name = Some(name);
        self.arguments.clear();
        Ok(())
    }

    pub(super) fn append(
        &mut self,
        on_event: &AgentEventEmitter,
        result: &mut StreamResult,
        event: &serde_json::Value,
    ) -> Result<(), String> {
        let delta = event["delta"].as_str().unwrap_or_default();
        if delta.is_empty() {
            return Ok(());
        }
        if !self.is_pending()
            || self.arguments.len().saturating_add(delta.len()) > MAX_TOOL_ARGUMENT_BYTES
        {
            return Err(invalid());
        }
        crate::services::agent_local::stream_buffer::record_generation_started(on_event, result);
        self.arguments.push_str(delta);
        Ok(())
    }

    pub(super) fn finish(
        &mut self,
        on_event: &AgentEventEmitter,
        result: &mut StreamResult,
        token_count: &mut u32,
        event: &serde_json::Value,
    ) -> Result<(), String> {
        let (Some(id), Some(name)) = (self.id.take(), self.name.take()) else {
            return Err(invalid());
        };
        if self.arguments.is_empty() {
            let full = event
                .pointer("/item/arguments")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(invalid)?;
            if full.len() > MAX_TOOL_ARGUMENT_BYTES {
                return Err(invalid());
            }
            self.arguments.push_str(full);
        }
        let arguments: serde_json::Value =
            serde_json::from_str(&self.arguments).map_err(|_| invalid())?;
        if !arguments.is_object() {
            return Err(invalid());
        }
        crate::services::agent_local::stream_buffer::record_tool_call_generation(
            on_event,
            result,
            &name,
            &arguments,
            token_count,
        );
        let _ = on_event.send(StreamEvent::ToolCall {
            name: name.clone(),
            domain: crate::services::agent_local::memory_tool::event_domain(&name, &arguments),
            arguments: arguments.clone(),
        });
        result.tool_calls.push((name, arguments));
        result.tool_call_ids.push(id);
        self.arguments.clear();
        Ok(())
    }
}

fn bounded(value: Option<&str>, max_bytes: usize) -> Result<String, String> {
    let value = value.ok_or_else(invalid)?;
    if value.is_empty() || value.len() > max_bytes || value.chars().any(char::is_control) {
        return Err(invalid());
    }
    Ok(value.to_string())
}

fn has_tool_capacity(result: &StreamResult) -> bool {
    result.tool_calls.len() < MAX_TOOL_CALLS
}

fn invalid() -> String {
    "provider_request_rejected".to_string()
}

#[cfg(test)]
#[path = "stream_tool_tests.rs"]
mod tests;
