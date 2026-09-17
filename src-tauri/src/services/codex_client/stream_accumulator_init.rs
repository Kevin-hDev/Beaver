use crate::services::compress::realtime_budget::RealtimeBudget;

use super::{
    limits::MAX_STREAM_TEXT_BYTES,
    stream_accumulator::{StreamAccumulator, StreamPolicy},
};

impl<'a> StreamAccumulator<'a> {
    pub(super) fn new(
        provider: &'a str,
        model: &'a str,
        tools: &'a [serde_json::Value],
        buffer_content: bool,
        realtime_budget: Option<RealtimeBudget>,
    ) -> Self {
        Self::new_with_capture(
            provider,
            model,
            tools,
            buffer_content,
            realtime_budget,
            None,
        )
    }

    pub(super) fn new_with_capture(
        provider: &'a str,
        model: &'a str,
        tools: &'a [serde_json::Value],
        buffer_content: bool,
        realtime_budget: Option<RealtimeBudget>,
        reasoning_capture: Option<crate::services::llm::reasoning_wire::ReasoningCapture>,
    ) -> Self {
        Self::new_with_limits(
            provider,
            model,
            tools,
            buffer_content,
            realtime_budget,
            reasoning_capture,
            StreamPolicy {
                max_text_bytes: MAX_STREAM_TEXT_BYTES,
                local_output_chars: None,
                accept_tools: true,
            },
        )
    }

    pub(super) fn new_silent(
        provider: &'a str,
        model: &'a str,
        tools: &'a [serde_json::Value],
        max_output_tokens: Option<u32>,
        max_text_bytes: usize,
    ) -> Self {
        Self::new_with_limits(
            provider,
            model,
            tools,
            true,
            None,
            None,
            StreamPolicy {
                max_text_bytes: max_text_bytes.min(MAX_STREAM_TEXT_BYTES),
                local_output_chars: max_output_tokens.map(|max| (max as usize).saturating_mul(6)),
                accept_tools: false,
            },
        )
    }
}
