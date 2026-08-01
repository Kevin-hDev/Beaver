use crate::services::agent_local::stream_buffer::StreamEventSink;
use crate::services::agent_local::types_ollama::{StreamOutcome, StreamResult};
use crate::services::compress::realtime_budget::RealtimeBudget;

use super::limits::MAX_STREAM_TEXT_BYTES;
use super::{replay::ReplayCollector, stream_protocol, stream_tool::StreamTool};

pub(super) struct StreamAccumulator<'a> {
    result: StreamResult,
    token_count: u32,
    text_bytes: usize,
    tool: StreamTool<'a>,
    replay: ReplayCollector,
    tools: &'a [serde_json::Value],
    buffer_content: bool,
    realtime_budget: Option<RealtimeBudget>,
}

impl<'a> StreamAccumulator<'a> {
    pub(super) fn new(
        tools: &'a [serde_json::Value],
        buffer_content: bool,
        realtime_budget: Option<RealtimeBudget>,
    ) -> Self {
        Self {
            result: StreamResult::default(),
            token_count: 0,
            text_bytes: 0,
            tool: StreamTool::new(tools),
            replay: ReplayCollector::default(),
            tools,
            buffer_content,
            realtime_budget,
        }
    }

    pub(super) fn apply(
        &mut self,
        on_event: &impl StreamEventSink,
        event: &serde_json::Value,
    ) -> Result<Option<StreamOutcome>, String> {
        match event["type"].as_str().unwrap_or_default() {
            "response.reasoning_summary_text.delta" => self.record_thinking(on_event, event)?,
            "response.output_text.delta" => return self.record_content(on_event, event),
            "response.output_item.added" => self.tool.start(on_event, &mut self.result, event)?,
            "response.function_call_arguments.delta" => {
                self.tool.append(on_event, &mut self.result, event)?
            }
            "response.output_item.done" => self.finish_item(on_event, event)?,
            "response.done" | "response.completed" => return self.completed(event).map(Some),
            "response.incomplete" => return Err(stream_protocol::incomplete_response()),
            "response.failed" | "error" => return Err(stream_protocol::failed_response(event)),
            _ => {}
        }
        Ok(None)
    }

    pub(super) fn has_partial_output(&self) -> bool {
        !self.result.content.is_empty()
            || !self.result.thinking.is_empty()
            || !self.result.tool_calls.is_empty()
    }

    fn record_thinking(
        &mut self,
        on_event: &impl StreamEventSink,
        event: &serde_json::Value,
    ) -> Result<(), String> {
        let delta = event["delta"].as_str().unwrap_or_default();
        if !delta.is_empty() {
            self.record_text_size(delta)?;
            crate::services::agent_local::stream_buffer::record_thinking(
                on_event,
                &mut self.result,
                delta.to_string(),
                &mut self.token_count,
            );
        }
        Ok(())
    }

    fn record_content(
        &mut self,
        on_event: &impl StreamEventSink,
        event: &serde_json::Value,
    ) -> Result<Option<StreamOutcome>, String> {
        let delta = event["delta"].as_str().unwrap_or_default();
        if delta.is_empty() {
            return Ok(None);
        }
        self.record_text_size(delta)?;
        crate::services::agent_local::stream_buffer::record_content(
            on_event,
            &mut self.result,
            delta.to_string(),
            &mut self.token_count,
            self.buffer_content,
        );
        if stream_protocol::should_interrupt(
            &mut self.realtime_budget,
            self.token_count,
            self.tool.is_pending() || !self.result.tool_calls.is_empty(),
        ) {
            self.attach_replay();
            return Ok(Some(StreamOutcome::InterruptedForCompression(
                std::mem::take(&mut self.result),
            )));
        }
        Ok(None)
    }

    fn finish_item(
        &mut self,
        on_event: &impl StreamEventSink,
        event: &serde_json::Value,
    ) -> Result<(), String> {
        let item = event
            .get("item")
            .ok_or_else(|| "provider_request_rejected".to_string())?;
        let mut replay_item = item.clone();
        super::replay::restore_tool_name(&mut replay_item, self.tools);
        self.replay
            .capture(&replay_item)
            .map_err(|_| "provider_payload_too_large".to_string())?;
        if item["type"].as_str() != Some("function_call") {
            return Ok(());
        }
        self.tool
            .finish(on_event, &mut self.result, &mut self.token_count, event)
    }

    fn completed(&mut self, event: &serde_json::Value) -> Result<StreamOutcome, String> {
        if self.tool.is_pending() {
            return Err("provider_request_rejected".to_string());
        }
        if let Some(usage) = event.pointer("/response/usage") {
            self.result.usage = crate::services::provider_usage::RequestUsage::from_json(usage);
            if let Some(usage) = &self.result.usage {
                self.result.prompt_tokens =
                    usage.input_tokens.and_then(|value| value.try_into().ok());
                self.result.eval_count =
                    usage.output_tokens.and_then(|value| value.try_into().ok());
            }
        }
        self.attach_replay();
        Ok(StreamOutcome::Completed(std::mem::take(&mut self.result)))
    }

    fn attach_replay(&mut self) {
        std::mem::take(&mut self.replay).attach(&mut self.result);
    }

    fn record_text_size(&mut self, delta: &str) -> Result<(), String> {
        self.text_bytes = self.text_bytes.saturating_add(delta.len());
        if self.text_bytes > MAX_STREAM_TEXT_BYTES {
            return Err("provider_payload_too_large".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "stream_accumulator_tests.rs"]
mod tests;
