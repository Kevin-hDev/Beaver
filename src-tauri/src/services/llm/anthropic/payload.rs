use serde_json::{json, Value};

use super::{messages, tools};
use crate::services::llm::stream_http::RequestConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) enum BuildError {
    InvalidImage,
    InvalidMaxTokens,
    InvalidMessage,
    InvalidReasoningBudget,
    InvalidReasoningMode,
    InvalidToolSchema,
    InvalidContinuation,
    TooManyImages,
    TooManyTools,
}

#[derive(Debug)]
pub(in crate::services::llm) struct PreparedPayload {
    pub payload: Value,
    pub replayed: Vec<crate::services::llm::reasoning_wire::replay::ReplayEvidence>,
}

pub(in crate::services::llm) fn build_payload(
    cfg: &RequestConfig<'_>,
    max_tokens: u32,
) -> Result<PreparedPayload, BuildError> {
    if max_tokens == 0 {
        return Err(BuildError::InvalidMaxTokens);
    }
    let mut converted = messages::convert(cfg.messages, cfg.tools)?;
    let replayed = crate::services::llm::reasoning_wire::replay::apply_anthropic_messages(
        cfg.messages,
        cfg.continuation_target,
        &mut converted.messages,
    )
    .map_err(|_| BuildError::InvalidContinuation)?;
    let mut payload = json!({
        "model": cfg.model,
        "messages": converted.messages,
        "max_tokens": max_tokens,
        "stream": true,
    });
    if !converted.system.is_empty() {
        payload["system"] = Value::Array(converted.system);
    }
    let native_tools = tools::convert(cfg.tools)?;
    if !native_tools.is_empty() {
        payload["tools"] = Value::Array(native_tools);
        payload["tool_choice"] = json!({"type": "auto"});
    }
    apply_thinking(
        &mut payload,
        cfg.model,
        cfg.think,
        cfg.reasoning_mode,
        max_tokens,
    )?;
    let cache = crate::services::llm::route_profile::cache_policy(cfg.provider_id, cfg.model)
        .ok_or(BuildError::InvalidMessage)?;
    crate::services::llm::prompt_cache_policy::apply_payload(&mut payload, cache, cfg.session_id);
    Ok(PreparedPayload { payload, replayed })
}

fn apply_thinking(
    payload: &mut Value,
    model: &str,
    think: bool,
    mode: Option<&str>,
    max_tokens: u32,
) -> Result<(), BuildError> {
    let contract = crate::services::llm::provider_model_lookup::resolve_local("anthropic", model);
    let selected = mode.map(str::to_string).or_else(|| {
        think.then(|| {
            contract
                .as_ref()
                .and_then(|value| value.default_reasoning_mode.clone())
                .unwrap_or_else(|| "medium".to_string())
        })
    });
    let mut selected = selected.unwrap_or_else(|| "off".to_string());
    if selected == "off"
        && contract.as_ref().is_some_and(|value| {
            value.supports_thinking && !value.reasoning_modes.iter().any(|mode| mode == "off")
        })
    {
        selected = contract
            .as_ref()
            .and_then(|value| value.default_reasoning_mode.clone())
            .ok_or(BuildError::InvalidReasoningMode)?;
    }
    let mode = crate::services::reasoning_continuity::contract::ReasoningModeId::from_name(Some(
        &selected,
    ))
    .ok_or(BuildError::InvalidReasoningMode)?;
    if mode == crate::services::reasoning_continuity::contract::ReasoningModeId::Off
        || !crate::services::reasoning_continuity::registry::reasoning_mode_is_live(
            crate::services::reasoning_continuity::contract::RouteId::Anthropic,
            model,
            mode,
        )
    {
        payload["thinking"] = json!({"type": "disabled"});
        return Ok(());
    }
    let adaptive = contract
        .as_ref()
        .is_some_and(|value| value.reasoning_modes.iter().any(|mode| mode == "auto"));
    if adaptive {
        if !contract
            .as_ref()
            .is_some_and(|value| value.reasoning_modes.iter().any(|mode| mode == &selected))
        {
            return Err(BuildError::InvalidReasoningMode);
        }
        // Les Claude 5 et les modèles adaptatifs récents omettent sinon le texte
        // visible tout en facturant et signant le raisonnement.
        payload["thinking"] = json!({"type": "adaptive", "display": "summarized"});
        if selected != "auto" {
            payload["output_config"] = json!({"effort": selected});
        }
        return Ok(());
    }
    let budget = match selected.as_str() {
        "low" => 1_024,
        "medium" => 4_096,
        "high" => 16_384,
        _ => return Err(BuildError::InvalidReasoningMode),
    };
    if budget >= max_tokens {
        return Err(BuildError::InvalidReasoningBudget);
    }
    payload["thinking"] = json!({"type": "enabled", "budget_tokens": budget});
    Ok(())
}
