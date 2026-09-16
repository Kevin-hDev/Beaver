use crate::services::agent_local::types_ollama::ChatMessage;
use serde_json::{json, Value};
use std::time::Duration;

use super::core_bridge::{CoreResponse, ExtensionBridgeError};

pub(super) async fn generate(
    context: &super::call_context::ExtensionCallContext,
    params: &Value,
) -> Result<CoreResponse, ExtensionBridgeError> {
    let scope = context
        .core_scope()
        .ok_or(ExtensionBridgeError::Context("core_context_required"))?;
    let session = crate::services::agent_local::session_store::get(&scope.agent.session_id)
        .await
        .map_err(|_| ExtensionBridgeError::Failed)?;
    let (prompt, max_tokens) = request_limits(params)?;
    let connection = optional_string(params, "connectionId").unwrap_or(&session.provider);
    let model = optional_string(params, "modelId").unwrap_or(&session.model);
    if !crate::services::llm::stream_dispatch::model_route_descriptor(connection)
        .is_some_and(|route| route.generation_supported)
    {
        return Err(ExtensionBridgeError::Backend("core_model_unavailable"));
    }
    if !super::core_model_catalog::is_configured(connection)
        || !super::core_model_catalog::known_model(connection, model).await
    {
        return Err(ExtensionBridgeError::Backend("core_model_unavailable"));
    }
    let _lease = super::core_model_quota::acquire(context.identity())
        .map_err(ExtensionBridgeError::Backend)?;
    let timeout = Duration::from_millis(super::types::MODEL_GENERATION_TIMEOUT_MS as u64);
    let purpose = crate::services::llm::request_purpose::RequestPurpose::for_request(
        &scope.agent.session_id,
        &scope.agent.request_id,
    )
    .await;
    let (result, ledger_recorded) = if connection == "ollama" {
        let (text, output, done_reason) =
            crate::services::agent_local::ollama_collect::collect_extension_text(
                model, prompt, timeout, max_tokens,
            )
            .await
            .map_err(|_| ExtensionBridgeError::Failed)?;
        (ollama_result(text, output, done_reason), false)
    } else {
        let messages = [ChatMessage {
            continuity_barrier_before: false,
            role: "user".to_string(),
            content: prompt.to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            display_thinking: None,
            continuation: None,
            tool_loop_reasoning: None,
        }];
        let result = crate::services::llm::text_generation::collect(
            connection,
            crate::services::llm::fast_mode::FastModeRequest::Unsupported,
            model,
            &messages,
            max_tokens,
            purpose,
            &scope.agent.session_id,
            &scope.agent.request_id,
            scope.agent.cancel.clone(),
            crate::services::provider_usage::UsageWorkload::Extension,
            timeout,
            timeout,
            super::types::MAX_MODEL_RESULT_BYTES,
            None,
        )
        .await
        .map_err(|_| ExtensionBridgeError::Failed)?;
        (result, true)
    };
    if result.content.len() > super::types::MAX_MODEL_RESULT_BYTES {
        return Err(ExtensionBridgeError::Failed);
    }
    Ok(CoreResponse::Json(json!({
        "text": result.content,
        "finishReason": finish_reason(result.done_reason.as_deref()),
        "usage": {
            "inputTokens": result.usage.as_ref().and_then(|usage| usage.input_tokens),
            "outputTokens": result.usage.as_ref().and_then(|usage| usage.output_tokens).or(result.eval_count.map(u64::from)),
            "ledgerRecorded": ledger_recorded,
        }
    })))
}

fn ollama_result(
    text: String,
    output: u32,
    done_reason: Option<String>,
) -> crate::services::agent_local::types_ollama::StreamResult {
    crate::services::agent_local::types_ollama::StreamResult {
        content: text,
        eval_count: Some(output),
        done_reason,
        ..Default::default()
    }
}

fn finish_reason(reason: Option<&str>) -> &'static str {
    match reason {
        Some("length" | "max_tokens") => "length",
        Some("content_filter") => "contentFilter",
        _ => "stop",
    }
}

fn required_string<'a>(params: &'a Value, key: &str) -> Result<&'a str, ExtensionBridgeError> {
    optional_string(params, key).ok_or(ExtensionBridgeError::Denied)
}

fn optional_string<'a>(params: &'a Value, key: &str) -> Option<&'a str> {
    params.get(key).and_then(Value::as_str).filter(|value| !value.is_empty())
}

fn request_limits(params: &Value) -> Result<(&str, u32), ExtensionBridgeError> {
    let prompt = required_string(params, "prompt")?;
    if prompt.len() > super::types::MAX_MODEL_PROMPT_BYTES {
        return Err(ExtensionBridgeError::Failed);
    }
    let max_tokens = params
        .get("maxOutputTokens")
        .and_then(Value::as_u64)
        .unwrap_or(super::types::MAX_MODEL_OUTPUT_TOKENS as u64);
    let max_tokens = u32::try_from(max_tokens)
        .ok()
        .filter(|value| *value > 0 && *value as usize <= super::types::MAX_MODEL_OUTPUT_TOKENS)
        .ok_or(ExtensionBridgeError::Failed)?;
    Ok((prompt, max_tokens))
}

#[cfg(test)]
mod tests {
    #[test]
    fn finish_reasons_match_contract() {
        for (wire, expected) in [
            (Some("stop"), "stop"),
            (Some("length"), "length"),
            (Some("content_filter"), "contentFilter"),
            (None, "stop"),
        ] {
            let value = super::finish_reason(wire);
            assert_eq!(value, expected);
            assert!(super::super::types::MODEL_FINISH_REASONS.contains(&value));
        }
    }

    #[test]
    fn ollama_length_reason_is_preserved() {
        let result = super::ollama_result("partial".into(), 10, Some("length".into()));
        assert_eq!(super::finish_reason(result.done_reason.as_deref()), "length");
    }

    #[test]
    fn model_limits_are_failures_not_permission_denials() {
        let oversized = "x".repeat(super::super::types::MAX_MODEL_PROMPT_BYTES + 1);
        for params in [
            serde_json::json!({"prompt": oversized}),
            serde_json::json!({"prompt": "ok", "maxOutputTokens": 0}),
        ] {
            assert_eq!(
                super::request_limits(&params),
                Err(super::ExtensionBridgeError::Failed)
            );
        }
    }

    #[tokio::test]
    async fn generation_preserves_route_purpose_and_usage() {
        let request_id = uuid::Uuid::new_v4().to_string();
        let _guard = crate::services::automations::actor_context::register_actor(
            &request_id,
            crate::services::automations::AutomationActor {
                origin: crate::services::automations::AutomationOrigin::Session,
                session_or_channel_id: "session".into(),
                current_automation_id: None,
            },
        )
        .unwrap();
        assert_eq!(
            crate::services::llm::request_purpose::RequestPurpose::for_request(
                "session",
                &request_id,
            )
            .await,
            crate::services::llm::request_purpose::RequestPurpose::Automation,
        );
        assert_eq!(
            serde_json::to_value(crate::services::provider_usage::UsageWorkload::Extension)
                .unwrap(),
            "extension",
        );
    }
}
