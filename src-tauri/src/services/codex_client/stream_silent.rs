use crate::services::agent_local::types_ollama::{ChatMessage, StreamResult};
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

use super::{http_error, request, stream::CODEX_IDLE_TIMEOUT_SECS};

pub async fn collect_chat_silent(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    think: bool,
    reasoning_mode: Option<&str>,
    max_output_tokens: Option<u32>,
    cancel: CancellationToken,
) -> Result<StreamResult, String> {
    let resp = if max_output_tokens.is_some() {
        request::post_codex_stream_with_timeout(
            model,
            messages,
            tools,
            think,
            reasoning_mode,
            std::time::Duration::from_secs(CODEX_IDLE_TIMEOUT_SECS),
        )
        .await?
    } else {
        request::post_codex_stream(model, messages, tools, think, reasoning_mode).await?
    };
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

    loop {
        let event = tokio::select! {
            _ = cancel.cancelled() => return Err("Annulé".to_string()),
            _ = tokio::time::sleep(idle_timeout) => {
                return Err(format!("Timeout Codex : {}s sans réponse", idle_timeout.as_secs()));
            }
            ev = sse.next() => match ev {
                Some(Ok(e)) => e,
                Some(Err(e)) => return Err(format!("SSE: {e}")),
                None => break,
            },
        };

        if event.data.trim() == "[DONE]" {
            break;
        }
        let parsed: serde_json::Value = match serde_json::from_str(&event.data) {
            Ok(v) => v,
            Err(_) => continue,
        };
        match parsed["type"].as_str().unwrap_or("") {
            "response.reasoning_summary_text.delta" => {
                result
                    .thinking
                    .push_str(parsed["delta"].as_str().unwrap_or(""));
            }
            "response.output_text.delta" => {
                result
                    .content
                    .push_str(parsed["delta"].as_str().unwrap_or(""));
                if output_is_over_local_limit(&result, max_output_tokens) {
                    break;
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
                break;
            }
            "response.failed" => return Err(http_error::stream_failure(&parsed)),
            _ => {}
        }
    }

    Ok(result)
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
