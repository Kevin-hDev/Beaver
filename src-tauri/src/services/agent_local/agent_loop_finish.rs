use super::agent_loop_completion;
use super::agent_loop_support;
use super::generation_metrics::GenerationAggregate;
use super::stream_events::AgentEventEmitter;
use super::types_ollama::{ChatMessage, StreamEvent};

pub type CompletionCounts = (Option<u32>, Option<u32>, Option<u32>, Option<u32>);

pub struct CompletedStreamTurn {
    event: StreamEvent,
    messages: Vec<ChatMessage>,
}

impl CompletedStreamTurn {
    pub fn with_messages(mut self, messages: Vec<ChatMessage>) -> Self {
        self.messages = messages;
        self
    }

    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    pub fn compression(messages: Vec<ChatMessage>) -> Self {
        Self {
            event: StreamEvent::Done {
                eval_count: None,
                eval_duration_ns: 0,
                final_tps: 0.0,
                tps_estimated: true,
                prompt_tokens: None,
                context_tokens: None,
            },
            messages,
        }
    }

    pub fn emit_done(self, on_event: &AgentEventEmitter) {
        let _ = on_event.send(self.event);
    }
}

pub fn emit_turn_end(on_event: &AgentEventEmitter, compressed_after_tools: bool) {
    if !compressed_after_tools {
        let _ = on_event.send(StreamEvent::TurnEnd {});
    }
}

pub async fn finish(
    counts: CompletionCounts,
    generation: GenerationAggregate,
    request: (&str, &str),
    ollama_model: Option<&str>,
) -> CompletedStreamTurn {
    let (event, _) =
        agent_loop_completion::done_event(counts.0, counts.1, counts.2, counts.3, generation);
    if let Some(model) = ollama_model {
        agent_loop_support::decharge_gpu(model).await;
    }
    let _ = request;
    CompletedStreamTurn {
        event,
        messages: Vec::new(),
    }
}
