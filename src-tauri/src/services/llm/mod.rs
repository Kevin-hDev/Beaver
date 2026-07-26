//! Module LLM multi-provider — abstraction unifiée OpenAI-compat.
//!
//! Tous les providers retenus (Groq, Gemini, Mistral, Cerebras, OpenRouter, OpenAI, DeepSeek)
//! exposent une API OpenAI-compatible. Un seul client (`openai_compat.rs`) les couvre tous
//! en changeant `base_url` et `api_key`.

pub mod agent_loop;
mod agent_loop_compression;
mod agent_loop_message;
mod agent_loop_request;
pub(crate) mod agent_loop_tools;
pub mod catalog;
pub mod compress_hook;
mod kimi_models;
#[cfg(test)]
mod kimi_models_tests;
pub mod model_pricing;
pub mod model_registry;
mod model_registry_refresh;
pub mod openai_compat;
mod openai_compat_models;
mod openai_compat_parsing;
#[cfg(test)]
mod openai_compat_parsing_tests;
pub mod provider_error;
pub(crate) mod providers;
pub mod registry_search;
pub(crate) mod request_purpose;
mod retry;
pub mod route;
pub mod runtime_models;
pub mod stream;
mod stream_chunk;
#[cfg(test)]
mod stream_chunk_tests;
mod stream_consume;
pub mod stream_convert;
mod stream_http;
mod stream_http_error;
mod stream_reasoning;
#[cfg(test)]
mod stream_reasoning_tests;
mod stream_silent;
mod stream_sse;
mod stream_tools;
mod timeouts;
pub mod tool_capable;
mod tool_schema;
pub mod types;
pub mod vision;

#[cfg(test)]
#[path = "sanitize_log_body_tests.rs"]
mod sanitize_log_body_tests;

pub(crate) fn sanitize_log_body(body: &str) -> String {
    let redacted = crate::services::agent_local::sensitive_data::redact_text(body);
    redacted
        .replace(|character: char| character.is_control(), " ")
        .chars()
        .take(200)
        .collect()
}

/// Helper non-streaming pour appels simples (utilisé par le scheduler heartbeat).
/// Retourne (contenu, tokens_totaux).
pub async fn collect_chat(
    provider_id: &str,
    model: &str,
    prompt: &str,
) -> Result<(String, u32), String> {
    if provider_id == "codex-oauth" {
        let msg = crate::services::agent_local::types_ollama::ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            reasoning_content: None,
        };
        let result = stream::collect_chat_silent(
            provider_id,
            model,
            &[msg],
            request_purpose::RequestPurpose::Automation,
            tokio_util::sync::CancellationToken::new(),
        )
        .await?;
        crate::services::provider_usage::record_automation(
            provider_id,
            model,
            result.usage.as_ref(),
        )
        .await;
        let total = crate::services::token_counting::sum_real_counts(
            result.prompt_tokens,
            result.eval_count,
        )
        .unwrap_or(0);
        return Ok((result.content, total));
    }
    let provider = openai_compat::OpenAiCompatProvider::new(provider_id).map_err(String::from)?;
    let req = types::ChatRequest {
        model: model.to_string(),
        messages: vec![types::ChatMessage {
            role: "user".to_string(),
            content: prompt.to_string(),
            tool_call_id: None,
            tool_calls: None,
        }],
        ..Default::default()
    };
    let resp = provider
        .chat_completion(req, request_purpose::RequestPurpose::Automation)
        .await
        .map_err(String::from)?;
    crate::services::provider_usage::record_automation(provider_id, model, Some(&resp.usage)).await;
    let total = resp
        .usage
        .total_tokens
        .or_else(
            || match (resp.usage.input_tokens, resp.usage.output_tokens) {
                (Some(input), Some(output)) => Some(input.saturating_add(output)),
                _ => None,
            },
        )
        .and_then(|value| value.try_into().ok())
        .unwrap_or(0);
    Ok((resp.content, total))
}
