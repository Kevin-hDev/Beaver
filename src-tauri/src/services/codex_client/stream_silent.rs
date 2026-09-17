#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use crate::services::agent_local::stream_buffer::DiscardStreamEvents;
use crate::services::agent_local::types_ollama::{ChatMessage, StreamOutcome, StreamResult};
use tokio_util::sync::CancellationToken;

use super::{
    request, stream_accumulator::StreamAccumulator, stream_measurement::StreamMeasurement,
};

pub async fn collect_chat_silent(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    fast_mode: crate::services::llm::fast_mode::FastModeRequest,
    max_output_tokens: Option<u32>,
    session_id: Option<&str>,
    cancel: CancellationToken,
    request_timeout: std::time::Duration,
    idle_timeout: std::time::Duration,
    max_text_bytes: usize,
    measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamResult, String> {
    let mut measurement = StreamMeasurement::new(measurement);
    let resp = request::post_codex_stream_with_timeout(
        model,
        messages,
        tools,
        reasoning_mode,
        session_id,
        fast_mode,
        request_timeout,
        &cancel,
    )
    .await?;
    measurement.mark_headers();
    consume_sse_silent_bounded(
        resp,
        cancel,
        idle_timeout,
        max_output_tokens,
        "openai",
        model,
        max_text_bytes,
        &mut measurement,
    )
    .await
}

pub(crate) async fn consume_external_responses_sse_silent(
    resp: reqwest::Response,
    cancel: CancellationToken,
    max_output_tokens: Option<u32>,
    provider: &str,
    model: &str,
    max_text_bytes: usize,
    measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamResult, String> {
    let mut measurement = StreamMeasurement::new(measurement);
    consume_sse_silent_bounded(
        resp,
        cancel,
        crate::services::compress::timeouts::compression_idle_timeout(),
        max_output_tokens,
        provider,
        model,
        max_text_bytes,
        &mut measurement,
    )
    .await
}

async fn consume_sse_silent_bounded(
    resp: reqwest::Response,
    cancel: CancellationToken,
    idle_timeout: std::time::Duration,
    max_output_tokens: Option<u32>,
    provider: &str,
    model: &str,
    max_text_bytes: usize,
    measurement: &mut StreamMeasurement<'_>,
) -> Result<StreamResult, String> {
    let accumulator =
        StreamAccumulator::new_silent(provider, model, &[], max_output_tokens, max_text_bytes);
    super::stream_reader::consume_sse_with_accumulator(
        &DiscardStreamEvents,
        resp,
        cancel,
        idle_timeout,
        accumulator,
        measurement,
    )
    .await
    .map(StreamOutcome::into_result)
}

#[cfg(test)]
#[path = "stream_silent_tests.rs"]
mod tests;
