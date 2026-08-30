//! Module LLM multi-provider — abstraction unifiée OpenAI-compat.
//!
//! Tous les providers retenus (Gemini, Mistral, Cerebras, OpenRouter, OpenAI, DeepSeek)
//! exposent une API OpenAI-compatible. Un seul client (`openai_compat.rs`) les couvre tous
//! en changeant `base_url` et `api_key`.

pub mod agent_loop;
mod agent_loop_compression;
mod agent_loop_message;
mod agent_loop_request;
mod agent_loop_request_types;
pub(crate) mod agent_loop_tools;
mod agent_loop_turn;
pub(crate) mod anthropic;
pub(crate) mod api_key_probe;
#[cfg(test)]
mod api_key_probe_tests;
pub mod catalog;
pub mod compress_hook;
mod endpoint_policy;
#[cfg(test)]
mod endpoint_policy_tests;
pub mod fast_mode;
mod kimi_models;
#[cfg(test)]
mod kimi_models_tests;
pub mod litellm_catalog;
mod litellm_catalog_lookup;
mod litellm_catalog_refresh;
pub mod litellm_catalog_search;
pub(crate) mod model_catalog;
#[cfg(test)]
mod model_catalog_tests;
mod model_metadata;
pub mod model_pricing;
pub mod openai_compat;
mod openai_compat_model_parser;
mod openai_compat_models;
mod openai_compat_parsing;
#[cfg(test)]
mod openai_compat_parsing_tests;
mod openai_responses;
mod openai_responses_reasoning;
pub(crate) mod prompt_cache_policy;
#[cfg(test)]
mod prompt_cache_policy_tests;
pub(crate) mod provider_diagnostics;
pub mod provider_error;
mod provider_model_capabilities;
#[cfg(test)]
mod provider_model_capabilities_tests;
pub(crate) mod provider_model_lookup;
pub(crate) mod provider_model_registry;
mod provider_model_registry_schema;
mod provider_model_registry_sources;
mod provider_model_registry_validation;
pub(crate) mod providers;
pub(crate) mod reasoning_wire;
mod request_auth;
#[cfg(test)]
mod request_auth_tests;
pub(crate) mod request_purpose;
mod retry;
pub mod route;
#[cfg(test)]
mod route_behavior_baseline_tests;
pub(crate) mod route_profile;
pub mod runtime_models;
pub mod stream;
mod stream_chunk;
#[cfg(test)]
mod stream_chunk_tests;
mod stream_consume;
mod stream_consume_budget;
mod stream_consume_record;
pub mod stream_convert;
pub(crate) mod stream_dispatch;
#[cfg(test)]
mod stream_dispatch_tests;
pub(crate) mod stream_fragments;
mod stream_http;
#[cfg(test)]
pub(crate) use stream_http::RequestConfig as RequestConfigForTest;
mod stream_http_error;
mod stream_http_payload;
#[cfg(test)]
pub(crate) use stream_http_payload::build_chat_payload as build_chat_payload_for_test;
#[cfg(test)]
mod legacy_capability_matrix_tests;
mod stream_http_send;
#[cfg(test)]
mod stream_http_send_tests;
mod stream_max_tokens;
mod stream_metrics;
pub(crate) mod stream_reasoning;
#[cfg(test)]
mod stream_reasoning_tests;
mod stream_silent;
mod stream_silent_consume;
pub(crate) mod stream_sse;
#[cfg(test)]
pub(crate) mod stream_test_transport;
mod stream_tools;
mod timeouts;
pub(crate) mod tool_schema;
mod tool_schema_names;
mod tool_schema_profile;
pub mod types;
pub mod vision;
#[cfg(test)]
mod wire_contract_tests;
mod xai_oauth_chat;
mod xai_oauth_payload;
mod xai_oauth_transport;
mod xai_oauth_transport_status;
#[cfg(test)]
mod xai_oauth_transport_tests;

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

pub(crate) fn context_usage_includes_reasoning(provider_id: &str) -> Option<bool> {
    route_profile::find(provider_id).map(|profile| profile.context_usage_includes_reasoning())
}

pub(crate) async fn model_context_length(provider_id: &str, model_id: &str) -> Option<u64> {
    let profile = route_profile::find(provider_id)?;
    if profile.client == route_profile::ClientSelector::Codex {
        let context = crate::services::codex_client::model_catalog::context_length(model_id).await;
        return (context > 0).then_some(context);
    }
    let canonical = profile.canonical_provider.as_str();
    if let Some(context) = provider_model_lookup::local_limits(canonical, model_id)
        .and_then(|limits| limits.context_window)
    {
        return Some(u64::from(context));
    }
    if let Some(context) =
        runtime_models::lookup(canonical, model_id).and_then(|model| model.context_length)
    {
        return Some(u64::from(context));
    }
    provider_model_lookup::limits(canonical, model_id)
        .await
        .and_then(|limits| limits.context_window)
        .map(u64::from)
}
