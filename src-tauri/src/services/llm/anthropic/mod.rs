mod client;
pub(super) mod messages;
pub(super) mod models;
mod payload;
mod stream;
mod stream_record;
#[allow(
    dead_code,
    reason = "continuation blocks are consumed by the next activation step"
)]
mod stream_state;
mod stream_state_limits;
mod stream_state_support;
pub(super) mod tools;
mod transport;

pub(super) use client::{list_models, test_connection};
pub(super) use payload::{build_payload, BuildError};
#[cfg(test)]
pub(super) use transport::collect_silent;
pub(super) use transport::{collect_silent_bounded, stream_chat};

pub(crate) fn prepared_context_count(
    messages: &[crate::services::agent_local::types_ollama::ChatMessage],
    provider_tools: &[serde_json::Value],
) -> crate::services::agent_local::context_usage_record::ContextTokenCount {
    let Ok(converted) = messages::convert(messages, provider_tools, None) else {
        return crate::services::agent_local::context_usage_record::ContextTokenCount {
            tokens: None,
            capacity_tokens: None,
            source: None,
            coverage:
                crate::services::agent_local::context_usage_record::ContextCountCoverage::Unknown,
        };
    };
    let Ok(tools) = tools::convert(provider_tools) else {
        return crate::services::agent_local::context_usage_record::ContextTokenCount {
            tokens: None,
            capacity_tokens: None,
            source: None,
            coverage:
                crate::services::agent_local::context_usage_record::ContextCountCoverage::Unknown,
        };
    };
    crate::services::agent_local::prepared_context_count::anthropic(&serde_json::json!({
        "system": converted.system,
        "messages": converted.messages,
        "tools": tools,
    }))
}

#[cfg(test)]
mod models_tests;
#[cfg(test)]
mod payload_tests;
#[cfg(test)]
mod stream_tests;
#[cfg(test)]
mod transport_tests;
