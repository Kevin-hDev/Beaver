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
    let payload_policy = super::route_profile::payload_policy(route.chat_provider_id, cfg.model)
        .expect("LlmRoute is constructed from a route profile");
    build_chat_payload_with_policy(cfg, route, max_tokens, payload_policy)
}

pub(super) fn build_chat_payload_with_policy(
    cfg: &RequestConfig<'_>,
    route: &LlmRoute,
    max_tokens: Option<u32>,
    payload_policy: super::route_profile::ResolvedPayloadPolicy,
) -> Result<PreparedChatPayload, super::reasoning_wire::replay::ReplayApplyError> {
    let provider_id = route.canonical_provider_id;
    let is_openrouter = super::openrouter_model_metadata::owns_catalog_metadata(provider_id);
    let catalog_model = is_openrouter
        .then(|| super::runtime_models::lookup(provider_id, cfg.model))
        .flatten();
    let cache_policy = super::route_profile::cache_policy(route.chat_provider_id, cfg.model)
        .expect("LlmRoute is constructed from a route profile");
    let mut messages = super::stream_convert::messages_to_openai_with_tools(
        cfg.messages,
        payload_policy.message,
        cfg.tools,
    );
    let supports_vision = crate::services::llm::provider_model_lookup::resolve_local(
        route.chat_provider_id,
        cfg.model,
    )
    .is_some_and(|capabilities| capabilities.supports_vision);
    super::tool_result_projection::append_openai_compatible_fallback(
        &mut messages,
        cfg.tool_result_previews,
        payload_policy.tool_result_media,
        supports_vision,
        payload_policy.message.images,
    );
    let mut payload = serde_json::json!({
        "model": cfg.model,
        "messages": messages,
        "stream": true,
    });
    if super::prompt_cache_policy::include_usage(cache_policy) {
        payload["stream_options"] = serde_json::json!({ "include_usage": true });
    }
    if let Some(value) = cfg.fast_mode.api_value() {
        payload["service_tier"] = value.into();
    }
    if let Some(max) = max_tokens {
        let output_limit_field = catalog_model.as_ref().map_or(
            payload_policy.output_limit_field,
            super::openrouter_request_contract::output_limit_field,
        );
        payload[output_limit_field] = max.into();
    }
    super::stream_reasoning::apply(
        &mut payload,
        payload_policy.parameters,
        cfg.model,
        cfg.think,
        cfg.reasoning_mode,
    );
    apply_tools(
        &mut payload,
        cfg,
        provider_id,
        payload_policy,
        is_openrouter,
        catalog_model.as_ref(),
    );
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
    is_openrouter: bool,
    catalog_model: Option<&super::types::ModelInfo>,
) {
    if cfg.tools.is_empty() {
        return;
    }
    let policy = super::route_profile::tool_policy(provider_id, cfg.model)
        .expect("LlmRoute is constructed from a route profile");
    let tools = super::tool_schema::tools_for_policy(policy.schema, policy.strict, cfg.tools);
    payload["tools"] = serde_json::Value::Array(tools);
    let supports = |parameter| {
        catalog_model.is_some_and(|model| {
            super::openrouter_request_contract::supports_parameter(model, parameter)
        })
    };
    if payload_policy.emit_tool_choice && (!is_openrouter || supports("tool_choice")) {
        payload["tool_choice"] = "auto".into();
    }
    if payload_policy.tool_stream {
        payload["tool_stream"] = true.into();
    }
    if payload_policy.parallel_tool_calls || (is_openrouter && supports("parallel_tool_calls")) {
        if payload_policy.parallel_tool_calls {
            payload["n"] = 1.into();
        }
        payload["parallel_tool_calls"] = true.into();
    }
}

#[cfg(test)]
#[path = "reasoning_wire/chat_contract_tests.rs"]
mod chat_contract_tests;

#[cfg(test)]
#[path = "reasoning_wire/structured_contract_tests.rs"]
mod structured_contract_tests;
