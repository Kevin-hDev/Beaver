#![expect(
    clippy::too_many_arguments,
    reason = "orchestration boundary keeps related runtime context explicit"
)]
use crate::services::agent_local::types_ollama::{ChatMessage, StreamResult};
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

use super::limits::MAX_STREAM_TEXT_BYTES;
use super::{request, stream_measurement::StreamMeasurement, stream_protocol};

pub async fn collect_chat_silent_for_compression(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    max_output_tokens: Option<u32>,
    session_id: Option<&str>,
    cancel: CancellationToken,
    measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<StreamResult, String> {
    let mut measurement = StreamMeasurement::new(measurement);
    let request_timeout = crate::services::compress::timeouts::compression_request_timeout();
    let idle_timeout = crate::services::compress::timeouts::compression_idle_timeout();
    let resp = request::post_codex_stream_with_timeout(
        model,
        messages,
        tools,
        reasoning_mode,
        session_id,
        request_timeout,
        &cancel,
    )
    .await?;
    measurement.mark_headers();
    consume_sse_silent(
        resp,
        cancel,
        idle_timeout,
        max_output_tokens,
        model,
        &mut measurement,
    )
    .await
}

async fn consume_sse_silent(
    resp: reqwest::Response,
    cancel: CancellationToken,
    idle_timeout: std::time::Duration,
    max_output_tokens: Option<u32>,
    model: &str,
    measurement: &mut StreamMeasurement<'_>,
) -> Result<StreamResult, String> {
    let mut sse = resp.bytes_stream().eventsource();
    let mut result = StreamResult::default();
    let mut text_bytes = 0_usize;

    loop {
        let event = tokio::select! {
            _ = cancel.cancelled() => return Err("Annulé".to_string()),
            _ = tokio::time::sleep(idle_timeout) => {
                return Err("provider_temporarily_unavailable".to_string());
            }
            ev = sse.next() => match ev {
                Some(Ok(e)) => e,
                Some(Err(_)) => return Err("provider_connection_failed".to_string()),
                None => return Err(stream_protocol::closed_before_completed()),
            },
        };

        if event.data.trim() == "[DONE]" {
            break;
        }
        let parsed = crate::services::llm::stream_sse::parse_json(&event.data)?;
        measurement.mark_first_event();
        match parsed["type"].as_str().unwrap_or("") {
            "response.reasoning_summary_text.delta" => {
                if parsed["delta"]
                    .as_str()
                    .is_some_and(|delta| !delta.is_empty())
                {
                    measurement.mark_first_useful();
                }
                append_bounded(
                    &mut result.thinking,
                    parsed["delta"].as_str().unwrap_or(""),
                    &mut text_bytes,
                )?;
            }
            "response.output_text.delta" => {
                if parsed["delta"]
                    .as_str()
                    .is_some_and(|delta| !delta.is_empty())
                {
                    measurement.mark_first_useful();
                }
                append_bounded(
                    &mut result.content,
                    parsed["delta"].as_str().unwrap_or(""),
                    &mut text_bytes,
                )?;
                if output_is_over_local_limit(&result, max_output_tokens) {
                    return Ok(result);
                }
            }
            "response.done" | "response.completed" => {
                if let Some(usage) = parsed.pointer("/response/usage") {
                    result.usage =
                        crate::services::provider_usage::RequestUsage::from_json_with_context(
                            usage,
                            crate::services::provider_usage::UsageContext::responses(
                                "openai", model,
                            ),
                        );
                    if let Some(usage) = &result.usage {
                        result.prompt_tokens =
                            usage.input_tokens.and_then(|value| value.try_into().ok());
                        result.eval_count =
                            usage.output_tokens.and_then(|value| value.try_into().ok());
                    }
                }
                return Ok(result);
            }
            "response.incomplete" => return Err(stream_protocol::incomplete_response()),
            "response.failed" | "error" => {
                return Err(stream_protocol::failed_response(&parsed));
            }
            _ => {}
        }
    }

    Err(stream_protocol::closed_before_completed())
}

fn append_bounded(target: &mut String, delta: &str, total: &mut usize) -> Result<(), String> {
    *total = total.saturating_add(delta.len());
    if *total > MAX_STREAM_TEXT_BYTES {
        return Err("provider_payload_too_large".to_string());
    }
    target.push_str(delta);
    Ok(())
}

fn output_is_over_local_limit(result: &StreamResult, max_output_tokens: Option<u32>) -> bool {
    let Some(max) = max_output_tokens else {
        return false;
    };
    result.content.chars().count() >= max as usize * 6
}

#[cfg(test)]
#[path = "stream_silent_tests.rs"]
mod tests;
