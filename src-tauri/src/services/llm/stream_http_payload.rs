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
    let mut payload = serde_json::json!({
        "model": cfg.model,
        "messages": super::stream_convert::messages_to_openai_with_tools(
            cfg.messages,
            provider_id,
            cfg.tools,
        ),
        "stream": true,
    });
    if super::prompt_cache_policy::include_usage(route) {
        payload["stream_options"] = serde_json::json!({ "include_usage": true });
    }
    if let Some(value) = cfg.fast_mode.api_value() {
        payload["service_tier"] = value.into();
    }
    if let Some(max) = max_tokens {
        let field = super::model_metadata::request_output_limit_field(provider_id, cfg.model);
        payload[field] = max.into();
    }
    super::stream_reasoning::apply(
        &mut payload,
        provider_id,
        cfg.model,
        cfg.think,
        cfg.reasoning_mode,
    );
    apply_tools(&mut payload, cfg, provider_id);
    if provider_id == "openrouter" {
        payload["provider"] = serde_json::json!({
            "require_parameters": true,
            "allow_fallbacks": true,
        });
    }
    super::prompt_cache_policy::apply_payload(&mut payload, route, cfg.model, cfg.session_id);
    let replayed = super::reasoning_wire::chat_text::apply_continuity(
        cfg.messages,
        cfg.continuation_target,
        &mut payload,
    )?;
    Ok(PreparedChatPayload { payload, replayed })
}

fn apply_tools(payload: &mut serde_json::Value, cfg: &RequestConfig<'_>, provider_id: &str) {
    if cfg.tools.is_empty() {
        return;
    }
    let tools = super::tool_schema::tools_for_provider(provider_id, cfg.model, cfg.tools);
    payload["tools"] = serde_json::Value::Array(tools);
    if provider_id != "deepseek" {
        // DeepSeek V4 choisit lui-même ses outils en mode thinking et rejette ce champ.
        payload["tool_choice"] = "auto".into();
    }
    if provider_id == "zai" {
        payload["tool_stream"] = true.into();
    }
}

#[cfg(test)]
#[path = "reasoning_wire/chat_contract_tests.rs"]
mod chat_contract_tests;

#[cfg(test)]
#[path = "reasoning_wire/structured_contract_tests.rs"]
mod structured_contract_tests;
