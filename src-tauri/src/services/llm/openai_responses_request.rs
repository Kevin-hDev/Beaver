use super::super::stream_http::{RequestConfig, RequestError};

#[cfg(test)]
pub(super) fn build_request(config: &RequestConfig<'_>) -> serde_json::Value {
    try_build_request(config).expect("a request without a continuation target cannot be rejected")
}

#[cfg(test)]
pub(super) fn try_build_request(
    config: &RequestConfig<'_>,
) -> Result<serde_json::Value, RequestError> {
    try_build_request_with_evidence(config).map(|prepared| prepared.body)
}

pub(super) struct PreparedResponseRequest {
    pub body: serde_json::Value,
    pub replayed: Vec<super::super::reasoning_wire::replay::ReplayEvidence>,
}

pub(super) fn try_build_request_with_evidence(
    config: &RequestConfig<'_>,
) -> Result<PreparedResponseRequest, RequestError> {
    let payload_policy =
        super::super::route_profile::payload_policy(config.provider_id, config.model)
            .ok_or(RequestError::InvalidConfiguration)?;
    let converted = crate::services::codex_client::convert::convert_messages_with_tools_and_continuity_evidence(
        config.messages,
        config.tools,
        config.continuation_target,
        payload_policy.message.tool_results,
    )
    .map_err(|_| RequestError::InvalidConfiguration)?;
    let mut input = converted.input;
    if payload_policy.tool_result_media == super::super::route_profile::ToolResultMedia::Inline {
        if let Some(previews) = config.tool_result_previews {
            if let Some(message) =
                super::super::tool_result_projection::responses_preview_input(previews)
            {
                input.push(message);
            }
        }
    }
    let tool_policy = super::super::route_profile::tool_policy(config.provider_id, config.model)
        .ok_or(RequestError::InvalidConfiguration)?;
    let mut body = serde_json::json!({
        "model": config.model,
        "instructions": converted.instructions,
        "input": input,
        "stream": true,
        "store": false,
        "tools": crate::services::codex_client::convert::convert_tools_to_responses_api(tool_policy, config.tools),
        "tool_choice": "auto",
        "parallel_tool_calls": false,
        "prompt_cache_key": super::super::prompt_cache_policy::routing_key(config.provider_id, config.model, config.session_id),
        "include": ["reasoning.encrypted_content"],
    });
    if let Some(tier) = config.fast_mode.api_value() {
        body["service_tier"] = tier.into();
    }
    if let Some(limit) = config.max_tokens {
        body[payload_policy.output_limit_field] = limit.into();
    }
    if let Some(effort) = super::super::openai_responses_reasoning::requested_effort(config) {
        body["reasoning"] = if effort == "none" {
            serde_json::json!({"effort": effort})
        } else {
            serde_json::json!({"effort": effort, "summary": "auto"})
        };
    }
    Ok(PreparedResponseRequest {
        body,
        replayed: converted.replayed,
    })
}
