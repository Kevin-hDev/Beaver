//! Module LLM multi-provider — abstraction unifiée OpenAI-compat.
//!
//! Tous les providers retenus (Groq, Gemini, Mistral, Cerebras, OpenRouter, OpenAI, DeepSeek)
//! exposent une API OpenAI-compatible. Un seul client (`openai_compat.rs`) les couvre tous
//! en changeant `base_url` et `api_key`.

pub mod agent_loop;
mod agent_loop_compression;
mod agent_loop_message;
mod agent_loop_request;
mod agent_loop_request_types;
pub(crate) mod agent_loop_tools;
mod agent_loop_turn;
pub mod catalog;
pub mod compress_hook;
pub mod fast_mode;
mod kimi_models;
#[cfg(test)]
mod kimi_models_tests;
pub mod litellm_catalog;
mod litellm_catalog_lookup;
mod litellm_catalog_refresh;
pub mod litellm_catalog_search;
mod model_metadata;
pub mod model_pricing;
pub mod openai_compat;
mod openai_compat_model_parser;
mod openai_compat_models;
mod openai_compat_parsing;
#[cfg(test)]
mod openai_compat_parsing_tests;
mod openai_responses;
pub(crate) mod prompt_cache_policy;
#[cfg(test)]
mod prompt_cache_policy_tests;
pub(crate) mod provider_diagnostics;
pub mod provider_error;
pub(crate) mod provider_model_lookup;
pub(crate) mod provider_model_registry;
mod provider_model_registry_sources;
mod provider_model_registry_validation;
pub(crate) mod providers;
pub(crate) mod reasoning_wire;
pub(crate) mod request_purpose;
mod retry;
pub mod route;
pub mod runtime_models;
pub mod stream;
mod stream_chunk;
#[cfg(test)]
mod stream_chunk_tests;
mod stream_consume;
mod stream_consume_budget;
pub mod stream_convert;
mod stream_http;
mod stream_http_error;
mod stream_http_payload;
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
pub mod tool_capable;
pub(crate) mod tool_schema;
mod tool_schema_names;
mod tool_schema_profile;
pub mod types;
pub mod vision;
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
