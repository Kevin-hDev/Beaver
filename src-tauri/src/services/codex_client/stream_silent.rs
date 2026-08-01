use crate::services::agent_local::types_ollama::{ChatMessage, StreamResult};
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

use super::{request, stream::CODEX_IDLE_TIMEOUT_SECS, stream_protocol};

const MAX_STREAM_TEXT_BYTES: usize = 32 * 1024 * 1024;

pub async fn collect_chat_silent(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    max_output_tokens: Option<u32>,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let resp = request::post_codex_stream(model, messages, tools, think, reasoning_mode).await?;
    consume_sse_silent(
        resp,
        cancel,
        std::time::Duration::from_secs(CODEX_IDLE_TIMEOUT_SECS),
        max_output_tokens,
    )
    .await
}

pub async fn collect_chat_silent_for_compression(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    max_output_tokens: Option<u32>,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let request_timeout = crate::services::compress::timeouts::compression_request_timeout();
    let idle_timeout = crate::services::compress::timeouts::compression_idle_timeout();
    let resp = request::post_codex_stream_with_timeout(
        model,
        messages,
        tools,
        think,
        reasoning_mode,
        request_timeout,
    )
    .await?;
    consume_sse_silent(resp, cancel, idle_timeout, max_output_tokens).await
}

async fn consume_sse_silent(
    resp: reqwest::Response,
    cancel: CancellationToken,
    idle_timeout: std::time::Duration,
    max_output_tokens: Option<u32>,
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
        let parsed: serde_json::Value = serde_json::from_str(&event.data)
            .map_err(|_| "provider_connection_failed".to_string())?;
        match parsed["type"].as_str().unwrap_or("") {
            "response.reasoning_summary_text.delta" => {
                append_bounded(
                    &mut result.thinking,
                    parsed["delta"].as_str().unwrap_or(""),
                    &mut text_bytes,
                )?;
            }
            "response.output_text.delta" => {
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
                    result.usage = crate::services::provider_usage::RequestUsage::from_json(usage);
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
            "response.failed" => return Err(stream_protocol::failed_response(&parsed)),
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
mod tests {
    use super::*;

    #[test]
    fn local_output_limit_is_optional() {
        let result = StreamResult {
            content: "x".repeat(100),
            ..Default::default()
        };
        assert!(!output_is_over_local_limit(&result, None));
    }

    #[test]
    fn local_output_limit_uses_safe_char_estimate() {
        let result = StreamResult {
            content: "x".repeat(60),
            ..Default::default()
        };
        assert!(output_is_over_local_limit(&result, Some(10)));
    }
}
