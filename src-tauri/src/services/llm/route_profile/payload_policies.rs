use super::policy_types::{ParameterPolicy, ToolChoicePolicy};
use super::{ClientSelector, MessageWirePolicy, ResolvedPayloadPolicy, RouteProfile};

pub(super) fn resolve(profile: &RouteProfile, model: &str) -> ResolvedPayloadPolicy {
    ResolvedPayloadPolicy {
        message: MessageWirePolicy {
            images: profile.wire.images,
            tool_results: profile.wire.tool_results,
            null_empty_tool_assistant: profile.policies.parameters != ParameterPolicy::DeepSeek,
            preserve_all_extra_content: profile.client == ClientSelector::Codex,
        },
        parameters: profile.policies.parameters,
        emit_tool_choice: profile.policies.tool_choice == ToolChoicePolicy::Default,
        tool_stream: profile.policies.parameters == ParameterPolicy::Zai,
        upstream_routing: profile.policies.schema == super::SchemaPolicy::Upstream,
        output_limit_field: output_limit_field(profile, model),
    }
}

fn output_limit_field(profile: &RouteProfile, model: &str) -> &'static str {
    if profile.policies.parameters == ParameterPolicy::Responses {
        return "max_output_tokens";
    }
    let completion_tokens = match profile.id.provider_id() {
        "openai" => super::super::providers::openai::uses_max_completion_tokens(model),
        "openrouter" => super::super::providers::openai::uses_max_completion_tokens(model),
        "moonshot" | "moonshot-oauth" => super::super::providers::moonshot::is_k3(model),
        _ => false,
    };
    if completion_tokens {
        "max_completion_tokens"
    } else {
        "max_tokens"
    }
}

#[cfg(test)]
pub(super) fn anthropic_fixture(
    max_tokens: Option<u32>,
) -> Result<serde_json::Value, &'static str> {
    let max_tokens = max_tokens.ok_or("provider_max_tokens_required")?;
    Ok(serde_json::json!({"max_tokens": max_tokens}))
}
