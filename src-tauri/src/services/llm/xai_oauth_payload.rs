use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm_oauth::XaiCatalogModel;

pub(super) struct PreparedResponsesPayload {
    pub payload: serde_json::Value,
    pub replayed: Vec<super::reasoning_wire::replay::ReplayEvidence>,
}

pub(super) fn build_with_evidence(
    model: &XaiCatalogModel,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    requested_mode: Option<&str>,
    session_id: Option<&str>,
    continuation_target: Option<
        &crate::services::reasoning_continuity::contract::ContinuationTarget,
    >,
) -> Result<PreparedResponsesPayload, String> {
    let payload_policy = super::route_profile::payload_policy("xai-oauth", &model.id)
        .ok_or_else(|| "provider_configuration_invalid".to_string())?;
    let tool_policy = super::route_profile::tool_policy("xai-oauth", &model.id)
        .ok_or_else(|| "provider_configuration_invalid".to_string())?;
    let converted =
        crate::services::codex_client::convert::convert_messages_with_tools_and_continuity_evidence(
            messages,
            tools,
            continuation_target,
            payload_policy.message.tool_results,
        )
        .map_err(|_| "reasoning_continuity_invalid".to_string())?;
    let effort = super::xai_oauth_transport::catalog_reasoning_mode(model, requested_mode);
    let tools =
        crate::services::codex_client::convert::convert_tools_to_responses_api(tool_policy, tools);
    let mut payload = serde_json::json!({
        "model": model.id,
        "instructions": converted.instructions,
        "input": converted.input,
        "stream": true,
        "store": false,
        "tools": tools,
        "tool_choice": "auto",
        "parallel_tool_calls": false,
        "prompt_cache_key": super::prompt_cache_policy::routing_key(
            "xai-oauth", &model.id, session_id,
        ),
        "include": ["reasoning.encrypted_content"],
    });
    if let Some(effort) = effort {
        payload["reasoning"] = serde_json::json!({"effort": effort});
    }
    Ok(PreparedResponsesPayload {
        payload,
        replayed: converted.replayed,
    })
}
