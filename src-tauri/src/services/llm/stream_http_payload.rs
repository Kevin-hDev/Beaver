use super::route::LlmRoute;
use super::stream_http::RequestConfig;

#[cfg(test)]
pub(crate) fn build_chat_payload(
    cfg: &RequestConfig<'_>,
    route: &LlmRoute,
    max_tokens: Option<u32>,
) -> Result<serde_json::Value, super::reasoning_wire::replay::ReplayApplyError> {
    build_chat_payload_with_evidence(cfg, route, max_tokens).map(|prepared| prepared.payload)
}

pub(super) struct PreparedChatPayload {
    pub payload: serde_json::Value,
    pub replayed: Vec<super::reasoning_wire::replay::ReplayEvidence>,
}

pub(super) fn build_chat_payload_with_evidence(
    cfg: &RequestConfig<'_>,
    route: &LlmRoute,
    max_tokens: Option<u32>,
) -> Result<PreparedChatPayload, super::reasoning_wire::replay::ReplayApplyError> {
    let provider_id = route.canonical_provider_id;
    let cache_policy = super::route_profile::cache_policy(route.chat_provider_id, cfg.model)
        .expect("LlmRoute is constructed from a route profile");
    let payload_policy = super::route_profile::payload_policy(route.chat_provider_id, cfg.model)
        .expect("LlmRoute is constructed from a route profile");
    let mut payload = serde_json::json!({
        "model": cfg.model,
        "messages": super::stream_convert::messages_to_openai_with_tools(
            cfg.messages,
            payload_policy.message,
            cfg.tools,
        ),
        "stream": true,
    });
    if super::prompt_cache_policy::include_usage(cache_policy) {
        payload["stream_options"] = serde_json::json!({ "include_usage": true });
    }
    if let Some(value) = cfg.fast_mode.api_value() {
        payload["service_tier"] = value.into();
    }
    if let Some(max) = max_tokens {
        payload[payload_policy.output_limit_field] = max.into();
    }
    super::stream_reasoning::apply(
        &mut payload,
        payload_policy.parameters,
        cfg.model,
        cfg.think,
        cfg.reasoning_mode,
    );
    apply_tools(&mut payload, cfg, provider_id, payload_policy);
    if payload_policy.upstream_routing {
        payload["provider"] = serde_json::json!({
            "require_parameters": true,
            "allow_fallbacks": true,
        });
    }
    super::prompt_cache_policy::apply_payload(&mut payload, cache_policy, cfg.session_id);
    let replayed = super::reasoning_wire::chat_text::apply_continuity(
        cfg.messages,
        cfg.continuation_target,
        &mut payload,
    )?;
    Ok(PreparedChatPayload { payload, replayed })
}

fn apply_tools(
    payload: &mut serde_json::Value,
    cfg: &RequestConfig<'_>,
    provider_id: &str,
    payload_policy: super::route_profile::ResolvedPayloadPolicy,
) {
    if cfg.tools.is_empty() {
        return;
    }
    let policy = super::route_profile::tool_policy(provider_id, cfg.model)
        .expect("LlmRoute is constructed from a route profile");
    let tools = super::tool_schema::tools_for_policy(policy.schema, policy.strict, cfg.tools);
    payload["tools"] = serde_json::Value::Array(tools);
    if payload_policy.emit_tool_choice {
        payload["tool_choice"] = "auto".into();
    }
    if payload_policy.tool_stream {
        payload["tool_stream"] = true.into();
    }
}

#[cfg(test)]
#[path = "reasoning_wire/chat_contract_tests.rs"]
mod chat_contract_tests;

#[cfg(test)]
#[path = "reasoning_wire/structured_contract_tests.rs"]
mod structured_contract_tests;
