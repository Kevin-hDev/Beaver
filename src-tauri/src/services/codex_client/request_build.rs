use super::convert;
use super::types::{CodexRequest, ReasoningConfig};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::fast_mode::FastModeRequest;

pub(super) fn build_codex_request(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    session_id: Option<&str>,
    fast_mode: FastModeRequest,
) -> CodexRequest {
    build_codex_request_with_continuity(
        model,
        messages,
        tools,
        reasoning_mode,
        session_id,
        fast_mode,
        None,
    )
    .expect("a request without a continuation target cannot be rejected")
}

pub(super) fn build_codex_request_with_continuity(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    session_id: Option<&str>,
    fast_mode: FastModeRequest,
    continuation_target: Option<
        &crate::services::reasoning_continuity::contract::ContinuationTarget,
    >,
) -> Result<CodexRequest, String> {
    build_codex_request_with_continuity_evidence(
        model,
        messages,
        tools,
        reasoning_mode,
        session_id,
        fast_mode,
        continuation_target,
    )
    .map(|prepared| prepared.body)
}

pub(super) struct PreparedCodexRequest {
    pub(super) body: CodexRequest,
    pub(super) replayed: Vec<crate::services::llm::reasoning_wire::replay::ReplayEvidence>,
}

pub(super) fn build_codex_request_with_continuity_evidence(
    model: &str,
    messages: &[ChatMessage],
    tools: &[serde_json::Value],
    reasoning_mode: Option<&str>,
    session_id: Option<&str>,
    fast_mode: FastModeRequest,
    continuation_target: Option<
        &crate::services::reasoning_continuity::contract::ContinuationTarget,
    >,
) -> Result<PreparedCodexRequest, String> {
    let payload_policy =
        crate::services::llm::route_profile::payload_policy(super::PROVIDER_ID, model)
            .ok_or_else(|| "provider_configuration_invalid".to_string())?;
    let tool_policy = crate::services::llm::route_profile::tool_policy(super::PROVIDER_ID, model)
        .ok_or_else(|| "provider_configuration_invalid".to_string())?;
    let converted = convert::convert_messages_with_tools_and_continuity_evidence(
        messages,
        tools,
        continuation_target,
        payload_policy.message.tool_results,
    )
    .map_err(|_| "reasoning_continuity_invalid".to_string())?;
    let converted_tools = convert::convert_tools_to_responses_api(tool_policy, tools);
    let body = CodexRequest {
        model: model.to_string(),
        instructions: converted.instructions,
        input: converted.input,
        stream: true,
        store: false,
        tools: converted_tools,
        tool_choice: "auto".to_string(),
        parallel_tool_calls: false,
        prompt_cache_key: crate::services::llm::prompt_cache_policy::routing_key(
            super::PROVIDER_ID,
            model,
            session_id,
        ),
        reasoning: Some(ReasoningConfig {
            effort: crate::services::reasoning::codex_effort(model, reasoning_mode),
            summary: "auto".to_string(),
        }),
        service_tier: fast_mode.codex_value().map(str::to_string),
        include: vec!["reasoning.encrypted_content".to_string()],
    };
    Ok(PreparedCodexRequest {
        body,
        replayed: converted.replayed,
    })
}
