use super::stream_events::AgentEventEmitter;
use super::types_ollama::StreamEvent;

#[derive(Clone)]
pub struct ShellProgress {
    emitter: AgentEventEmitter,
    tool_call_index: usize,
}

impl ShellProgress {
    pub fn new(emitter: AgentEventEmitter, tool_call_index: usize) -> Self {
        Self {
            emitter,
            tool_call_index,
        }
    }

    pub fn emit(&self, content: &str, elapsed_ms: u64) {
        let content = super::sensitive_data::redact_text(content);
        let _ = self.emitter.send(StreamEvent::ToolOutput {
            tool_call_index: self.tool_call_index,
            content,
            elapsed_ms,
        });
    }
}
