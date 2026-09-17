use super::stream_chunk::{self, ParsedChunk};
use super::stream_tools::ToolCallAccumulator;
use crate::services::agent_local::stream_buffer::StreamEventSink;
use crate::services::agent_local::types_ollama::{StreamEvent, StreamResult};
use crate::services::stream_utils::{FilteredChunk, ThinkTagFilter};

pub(super) enum OutputMode {
    Interactive { buffer_content: bool },
    Silent { max_text_bytes: usize },
}

pub(super) struct ChatStreamAccumulator {
    result: StreamResult,
    token_count: u32,
    tools: ToolCallAccumulator,
    think_filter: ThinkTagFilter,
    fragments: super::stream_fragments::StreamFragmentState,
    mode: OutputMode,
}

impl ChatStreamAccumulator {
    pub(super) fn new(fragment_mode: super::route_profile::FragmentMode, mode: OutputMode) -> Self {
        Self {
            result: StreamResult::default(),
            token_count: 0,
            tools: ToolCallAccumulator::new(),
            think_filter: ThinkTagFilter::new(),
            fragments: super::stream_fragments::StreamFragmentState::new(fragment_mode),
            mode,
        }
    }

    pub(super) fn has_pending_tools(&self) -> bool {
        self.tools.has_pending()
    }

    pub(super) fn output_tokens(&self) -> u32 {
        self.token_count
    }

    pub(super) fn apply(
        &mut self,
        on_event: &impl StreamEventSink,
        value: &serde_json::Value,
        usage_context: crate::services::provider_usage::UsageContext<'_>,
        error_policy: super::route_profile::ErrorPolicy,
    ) -> Result<bool, String> {
        let mut useful = false;
        for chunk in stream_chunk::parse_value_with_context(value, usage_context) {
            match chunk {
                ParsedChunk::Thinking(thinking) => {
                    let thinking = self.fragments.thinking(&thinking)?;
                    if thinking.is_empty() || matches!(self.mode, OutputMode::Silent { .. }) {
                        continue;
                    }
                    useful = true;
                    crate::services::agent_local::stream_buffer::record_thinking(
                        on_event,
                        &mut self.result,
                        thinking,
                        &mut self.token_count,
                    );
                }
                ParsedChunk::Content(content) => {
                    let content = self.fragments.content(&content)?;
                    if content.is_empty() {
                        continue;
                    }
                    useful = true;
                    if matches!(self.mode, OutputMode::Interactive { .. }) {
                        crate::services::agent_local::stream_buffer::record_generation_started(
                            on_event,
                            &mut self.result,
                        );
                    }
                    for filtered in self.think_filter.feed(&content) {
                        self.record_filtered(on_event, filtered)?;
                    }
                }
                ParsedChunk::ToolCalls(tool_calls) => {
                    if !tool_calls.is_empty() {
                        useful = true;
                        if matches!(self.mode, OutputMode::Interactive { .. }) {
                            crate::services::agent_local::stream_buffer::record_generation_started(
                                on_event,
                                &mut self.result,
                            );
                        }
                    }
                    self.tools.ingest(&tool_calls);
                }
                ParsedChunk::Usage(usage) => {
                    self.result.eval_count =
                        usage.output_tokens.and_then(|value| value.try_into().ok());
                    self.result.prompt_tokens =
                        usage.context_input_tokens(usage_context.api_format);
                    self.result.usage = Some(usage);
                }
                ParsedChunk::GenerationDuration(duration_ns) => {
                    if matches!(self.mode, OutputMode::Interactive { .. }) {
                        self.result.generation.record_native_duration(duration_ns);
                    }
                }
                ParsedChunk::FinishReason(reason) => self.result.done_reason = Some(reason.into()),
                ParsedChunk::ProviderError(status) => {
                    return Err(stream_chunk::provider_error_code(error_policy, status).to_string());
                }
            }
        }
        Ok(useful)
    }

    pub(super) fn finish(
        mut self,
        on_event: &impl StreamEventSink,
        provider_id: &str,
        provider_tools: &[serde_json::Value],
        interrupted: bool,
    ) -> Result<StreamResult, String> {
        for chunk in self.think_filter.flush() {
            self.record_filtered(on_event, chunk)?;
        }
        if super::stream_completion::terminal_error(&self.result).is_some() {
            self.tools = ToolCallAccumulator::new();
        }
        self.finalize_tools(on_event, provider_id, provider_tools);
        if !interrupted {
            super::stream_completion::finish(&mut self.result);
        }
        Ok(self.result)
    }

    fn record_filtered(
        &mut self,
        on_event: &impl StreamEventSink,
        chunk: FilteredChunk,
    ) -> Result<(), String> {
        match (&self.mode, chunk) {
            (OutputMode::Silent { .. }, FilteredChunk::Thinking(_)) => {}
            (OutputMode::Silent { max_text_bytes }, FilteredChunk::Content(content)) => {
                if self.result.content.len().saturating_add(content.len()) > *max_text_bytes {
                    return Err("provider_payload_too_large".to_string());
                }
                self.result.content.push_str(&content);
            }
            (OutputMode::Interactive { .. }, FilteredChunk::Thinking(content)) => {
                crate::services::agent_local::stream_buffer::record_thinking(
                    on_event,
                    &mut self.result,
                    content,
                    &mut self.token_count,
                );
            }
            (OutputMode::Interactive { buffer_content }, FilteredChunk::Content(content)) => {
                crate::services::agent_local::stream_buffer::record_content(
                    on_event,
                    &mut self.result,
                    content,
                    &mut self.token_count,
                    *buffer_content,
                );
            }
        }
        Ok(())
    }

    fn finalize_tools(
        &mut self,
        on_event: &impl StreamEventSink,
        provider_id: &str,
        provider_tools: &[serde_json::Value],
    ) {
        let (tool_calls, ids, extra_content) = std::mem::take(&mut self.tools).finalize();
        for (index, (wire_name, arguments)) in tool_calls.iter().enumerate() {
            let name = if matches!(self.mode, OutputMode::Interactive { .. }) {
                super::tool_schema::restore_tool_name_for_provider(
                    provider_id,
                    wire_name,
                    provider_tools,
                )
            } else {
                wire_name.clone()
            };
            if matches!(self.mode, OutputMode::Interactive { .. }) {
                crate::services::agent_local::stream_buffer::record_tool_call_generation(
                    on_event,
                    &mut self.result,
                    &name,
                    arguments,
                    &mut self.token_count,
                );
                let _ = on_event.send_event(StreamEvent::ToolCall {
                    name: name.clone(),
                    arguments: arguments.clone(),
                    tool_call_index: index,
                    tool_call_id: ids.get(index).cloned(),
                    domain: crate::services::agent_local::memory_tool::event_domain(
                        &name, arguments,
                    ),
                    extra_content: extra_content.get(index).cloned().flatten(),
                });
            }
            self.result.tool_calls.push((name, arguments.clone()));
            if let Some(id) = ids.get(index) {
                self.result.tool_call_ids.push(id.clone());
            }
            self.result
                .tool_call_extra_content
                .push(extra_content.get(index).cloned().flatten());
        }
    }
}
