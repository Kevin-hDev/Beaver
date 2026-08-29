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
    #[allow(
        dead_code,
        reason = "populated by the continuity adapter before activation"
    )]
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
    let reasoning_mode = cfg
        .think
        .then_some(cfg.reasoning_mode)
        .flatten()
        .or(Some("off"));
    apply_thinking(&mut payload, reasoning_mode, max_tokens)?;
    let cache = crate::services::llm::route_profile::cache_policy(cfg.provider_id, cfg.model)
        .ok_or(BuildError::InvalidMessage)?;
    crate::services::llm::prompt_cache_policy::apply_payload(&mut payload, cache, cfg.session_id);
    Ok(PreparedPayload { payload, replayed })
}

fn apply_thinking(
    payload: &mut Value,
    mode: Option<&str>,
    max_tokens: u32,
) -> Result<(), BuildError> {
    let budget = match mode.unwrap_or("medium") {
        "off" => {
            payload["thinking"] = json!({"type": "disabled"});
            return Ok(());
        }
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
