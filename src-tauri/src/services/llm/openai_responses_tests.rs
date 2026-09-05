use super::{build_request, try_build_request, try_build_request_with_evidence};
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::fast_mode::FastModeRequest;
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::llm::stream_http::RequestConfig;

fn request<'a>(
    messages: &'a [ChatMessage],
    tools: &'a [serde_json::Value],
    reasoning_mode: Option<&'a str>,
    fast_mode: FastModeRequest,
) -> RequestConfig<'a> {
    RequestConfig {
        provider_id: "openai",
        model: "gpt-5.6-luna",
        messages,
        tools,
        think: reasoning_mode != Some("off"),
        reasoning_mode,
        max_tokens: None,
        purpose: RequestPurpose::ManualChat,
        session_id: Some("session-fixture"),
        fast_mode,
        tool_result_previews: None,
        continuation_target: None,
    }
}

fn xai_request<'a>(
    messages: &'a [ChatMessage],
    tools: &'a [serde_json::Value],
) -> RequestConfig<'a> {
    RequestConfig {
        provider_id: "xai",
        model: "grok-4.6",
        messages,
        tools,
        think: true,
        reasoning_mode: Some("high"),
        max_tokens: None,
        purpose: RequestPurpose::ManualChat,
        session_id: Some("xai-fixture"),
        fast_mode: FastModeRequest::Unsupported,
        tool_result_previews: None,
        continuation_target: None,
    }
}

fn preview_batch() -> crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch {
    crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch::from_ephemeral(
        2,
        Some("call-preview".into()),
        preview_artifact(),
    )
}

fn preview_artifact() -> crate::services::agent_local::tool_artifact::EphemeralArtifact {
    crate::services::agent_local::tool_artifact::EphemeralArtifact {
        metadata: crate::services::agent_local::tool_artifact::ArtifactMetadata {
            name: "preview.png".into(),
            mime_type: "image/png".into(),
            bytes: 8,
            sha256: "a".repeat(64),
            purpose: crate::services::agent_local::tool_artifact::ArtifactPurpose::Preview,
            source:
                crate::services::agent_local::tool_artifact::ArtifactSource::ExtensionResource {
                    resource_id: "extension:demo:preview".into(),
                    catalog_fingerprint: "b".repeat(64),
                },
        },
        bytes: b"\x89PNG\r\n\x1a\n".to_vec(),
    }
}

fn fixture_target(
    scope: &str,
) -> crate::services::reasoning_continuity::contract::ContinuationTarget {
    use crate::services::reasoning_continuity::contract::{
        ContinuationTarget, ContinuationUse, CredentialScope, ReasoningModeId, ReplayTarget,
        RouteId,
    };
    ContinuationTarget::FixtureCandidate(ReplayTarget {
        route_id: RouteId::OpenAi,
        model_id: "gpt-5.6-luna".into(),
        credential_scope: CredentialScope::authenticated(scope).unwrap(),
        reasoning_mode: ReasoningModeId::Medium,
        continuation_use: ContinuationUse::UserContinuation,
    })
}

fn xai_fixture_target(
    scope: &str,
) -> crate::services::reasoning_continuity::contract::ContinuationTarget {
    use crate::services::reasoning_continuity::contract::{
        ContinuationTarget, ContinuationUse, CredentialScope, ReasoningModeId, ReplayTarget,
        RouteId,
    };
    ContinuationTarget::FixtureCandidate(ReplayTarget {
        route_id: RouteId::Xai,
        model_id: "grok-4.6".into(),
        credential_scope: CredentialScope::authenticated(scope).unwrap(),
        reasoning_mode: ReasoningModeId::High,
        continuation_use: ContinuationUse::UserContinuation,
    })
}

fn native_assistant(
    target: &crate::services::reasoning_continuity::contract::ContinuationTarget,
) -> ChatMessage {
    let replay = target.replay().unwrap();
    ChatMessage::assistant(
        "visible answer".into(),
        None,
        Some(crate::services::reasoning_continuity::envelope::ReasoningEnvelope::new(
            crate::services::reasoning_continuity::contract::ContractId::OpenAiResponsesV1,
            crate::services::reasoning_continuity::envelope::ReasoningSource::from_target(replay),
            crate::services::reasoning_continuity::envelope::CompletionState::Complete,
            crate::services::reasoning_continuity::envelope::ContinuationState::ResponsesLocal {
                items: vec![
                    serde_json::json!({"type":"reasoning","encrypted_content":"opaque"}),
                    serde_json::json!({"type":"message","content":[]}),
                ],
            },
            Vec::new(),
        )),
        None,
        None,
    )
}

fn xai_native_assistant(
    target: &crate::services::reasoning_continuity::contract::ContinuationTarget,
) -> ChatMessage {
    let replay = target.replay().unwrap();
    ChatMessage::assistant(
        "visible answer".into(),
        None,
        Some(crate::services::reasoning_continuity::envelope::ReasoningEnvelope::new(
            crate::services::reasoning_continuity::contract::ContractId::XaiResponsesV1,
            crate::services::reasoning_continuity::envelope::ReasoningSource::from_target(replay),
            crate::services::reasoning_continuity::envelope::CompletionState::Complete,
            crate::services::reasoning_continuity::envelope::ContinuationState::ResponsesLocal {
                items: vec![serde_json::json!({
                    "type":"reasoning",
                    "encrypted_content":"opaque-xai"
                })],
            },
            Vec::new(),
        )),
        None,
        None,
    )
}

#[test]
fn api_request_uses_responses_reasoning_and_fast_contract() {
    let messages = [ChatMessage::user("bonjour".into())];
    let body = build_request(&request(
        &messages,
        &[],
        Some("medium"),
        FastModeRequest::Fast,
    ));

    assert_eq!(body["model"], "gpt-5.6-luna");
    assert_eq!(body["reasoning"]["effort"], "medium");
    assert_eq!(body["reasoning"]["summary"], "auto");
    assert_eq!(body["service_tier"], "fast");
    assert_eq!(body["stream"], true);
    assert_eq!(body["store"], false);
    assert!(body.get("reasoning_effort").is_none());
    assert!(body.get("messages").is_none());
}

#[test]
fn responses_payload_receives_verified_preview_with_its_original_tool_call_id() {
    let messages = [ChatMessage::tool(
        "done".into(),
        Some("call-preview".into()),
        None,
    )];
    let previews = preview_batch();
    let config = RequestConfig {
        provider_id: "openai",
        model: "gpt-5.6-luna",
        messages: &messages,
        tools: &[],
        think: false,
        reasoning_mode: None,
        max_tokens: None,
        purpose: RequestPurpose::ManualChat,
        session_id: Some("session-preview"),
        fast_mode: FastModeRequest::Unsupported,
        tool_result_previews: Some(&previews),
        continuation_target: None,
    };

    let body = build_request(&config);
    let preview = body["input"].as_array().unwrap().last().unwrap();
    assert_eq!(preview["role"], "user");
    assert_eq!(
        preview["content"][1]["text"],
        "Extension preview for tool call call-preview (index 2): preview.png"
    );
    assert_eq!(
        preview["content"][2]["image_url"],
        "data:image/png;base64,iVBORw0KGgo="
    );
}

#[test]
fn xai_text_only_route_does_not_receive_preview_bytes() {
    let messages = [ChatMessage::tool(
        "done".into(),
        Some("call-preview".into()),
        None,
    )];
    let previews = preview_batch();
    let mut config = xai_request(&messages, &[]);
    config.tool_result_previews = Some(&previews);

    assert!(!build_request(&config).to_string().contains("data:image"));
}

#[test]
fn xai_api_request_uses_responses_and_encrypted_reasoning() {
    let messages = [ChatMessage::user("bonjour".into())];
    let body = build_request(&xai_request(&messages, &[]));

    assert_eq!(body["model"], "grok-4.6");
    assert_eq!(body["reasoning"]["effort"], "high");
    assert_eq!(
        body["include"],
        serde_json::json!(["reasoning.encrypted_content"])
    );
    assert_eq!(body["store"], false);
    assert!(body.get("messages").is_none());
}

#[test]
fn api_request_preserves_tool_continuation_in_responses_shape() {
    let messages = [
        ChatMessage::assistant(
            String::new(),
            None,
            None,
            None,
            Some(vec![
                crate::services::agent_local::types_ollama::ToolCallOllama {
                    id: Some("call_1".into()),
                    function: crate::services::agent_local::types_ollama::ToolCallFunction {
                        name: "lookup".into(),
                        arguments: serde_json::json!({"city": "Paris"}),
                    },
                    extra_content: None,
                },
            ]),
        ),
        ChatMessage::tool("18 C".into(), Some("call_1".into()), None),
    ];
    let tools = [serde_json::json!({
        "type": "function",
        "function": {
            "name": "lookup",
            "description": "Lookup weather",
            "parameters": {"type": "object", "properties": {}}
        }
    })];
    let body = build_request(&request(
        &messages,
        &tools,
        Some("high"),
        FastModeRequest::Standard,
    ));

    assert_eq!(body["service_tier"], "default");
    assert_eq!(body["tools"][0]["type"], "function");
    assert_eq!(body["input"][0]["type"], "function_call");
    assert_eq!(body["input"][1]["type"], "function_call_output");
}

#[test]
fn unsupported_fast_mode_omits_service_tier() {
    let messages = [ChatMessage::user("bonjour".into())];
    let body = build_request(&request(
        &messages,
        &[],
        Some("medium"),
        FastModeRequest::Unsupported,
    ));

    assert!(body.get("service_tier").is_none());
}

#[test]
fn responses_continuity_replays_native_items_at_the_assistant_position_without_tools() {
    let target = fixture_target("openai-scope");
    let messages = [
        native_assistant(&target),
        ChatMessage::user("continue".into()),
    ];
    let mut config = request(&messages, &[], Some("medium"), FastModeRequest::Standard);
    config.continuation_target = Some(&target);

    let prepared = try_build_request_with_evidence(&config).unwrap();
    let body = prepared.body;

    assert_eq!(body["input"][0]["type"], "reasoning");
    assert_eq!(body["input"][0]["encrypted_content"], "opaque");
    assert_eq!(body["input"][1]["type"], "message");
    assert_eq!(body["input"][2]["role"], "user");
    assert_eq!(prepared.replayed.len(), 1);
}

#[test]
fn responses_continuity_blocks_wrong_scope_and_required_missing_state() {
    let target = fixture_target("openai-scope");
    let messages = [
        native_assistant(&target),
        ChatMessage::user("continue".into()),
    ];
    let wrong = fixture_target("other-openai-scope");
    let mut config = request(&messages, &[], Some("medium"), FastModeRequest::Standard);
    config.continuation_target = Some(&wrong);
    assert!(try_build_request(&config).is_err());

    let missing = [
        ChatMessage::assistant("visible".into(), None, None, None, None),
        ChatMessage::user("continue".into()),
    ];
    let mut config = request(&missing, &[], Some("medium"), FastModeRequest::Standard);
    config.continuation_target = Some(&target);
    assert!(try_build_request(&config).is_err());
}

#[test]
fn responses_continuity_ignores_required_state_before_a_migration_barrier() {
    let target = fixture_target("openai-scope");
    let old = ChatMessage::assistant("legacy answer".into(), None, None, None, None);
    let mut current = ChatMessage::user("continue".into());
    current.continuity_barrier_before = true;
    let messages = [old, current];
    let mut config = request(&messages, &[], Some("medium"), FastModeRequest::Standard);
    config.continuation_target = Some(&target);

    assert!(try_build_request(&config).is_ok());
}

#[test]
fn xai_responses_continuity_replays_native_items() {
    let target = xai_fixture_target("xai-scope");
    let messages = [
        xai_native_assistant(&target),
        ChatMessage::user("continue".into()),
    ];
    let mut config = xai_request(&messages, &[]);
    config.continuation_target = Some(&target);

    let prepared = try_build_request_with_evidence(&config).expect("xAI native replay");
    let body = prepared.body;

    assert_eq!(body["input"][0]["type"], "reasoning");
    assert_eq!(body["input"][0]["encrypted_content"], "opaque-xai");
    assert_eq!(prepared.replayed.len(), 1);
}

#[test]
fn xai_responses_blocks_wrong_scope_and_required_missing_state() {
    let target = xai_fixture_target("xai-scope");
    let messages = [
        xai_native_assistant(&target),
        ChatMessage::user("continue".into()),
    ];
    let wrong = xai_fixture_target("other-xai-scope");
    let mut config = xai_request(&messages, &[]);
    config.continuation_target = Some(&wrong);
    assert!(try_build_request(&config).is_err());

    let missing = [
        ChatMessage::assistant("visible".into(), None, None, None, None),
        ChatMessage::user("continue".into()),
    ];
    let mut config = xai_request(&missing, &[]);
    config.continuation_target = Some(&target);
    assert!(try_build_request(&config).is_err());
}

#[tokio::test]
async fn runtime_dispatch_cannot_fall_back_to_chat_completions() {
    let session_id = "openai-responses-runtime";
    let scenario = crate::services::llm::stream_test_transport::StreamScenario::start(
        session_id,
        [crate::services::llm::stream_test_transport::ScriptedResponse::Success],
    )
    .await;
    let emitter = crate::services::agent_local::stream_events::AgentEventEmitter::test(
        session_id.to_string(),
    );
    let messages = [ChatMessage::user("bonjour".into())];
    let previews =
        crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch::default();

    crate::services::llm::stream::stream_chat_no_done(
        &emitter,
        session_id,
        "request-responses-runtime",
        0,
        1,
        "openai",
        FastModeRequest::Fast,
        RequestPurpose::ManualChat,
        "gpt-5.6-luna",
        &messages,
        &[],
        true,
        Some("medium"),
        &previews,
        tokio_util::sync::CancellationToken::new(),
        false,
        None,
        None,
        None,
    )
    .await
    .expect("Responses stream completes");

    let payloads = scenario.payloads();
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0]["reasoning"]["effort"], "medium");
    assert_eq!(payloads[0]["service_tier"], "fast");
    assert!(payloads[0].get("reasoning_effort").is_none());
    assert!(payloads[0].get("messages").is_none());
}

#[tokio::test]
async fn xai_runtime_dispatch_cannot_fall_back_to_chat_completions() {
    let session_id = "xai-responses-runtime";
    let scenario = crate::services::llm::stream_test_transport::StreamScenario::start(
        session_id,
        [crate::services::llm::stream_test_transport::ScriptedResponse::Success],
    )
    .await;
    let emitter = crate::services::agent_local::stream_events::AgentEventEmitter::test(
        session_id.to_string(),
    );
    let messages = [ChatMessage::user("bonjour".into())];
    let previews =
        crate::services::agent_local::tool_artifact_preview::ToolResultPreviewBatch::default();

    crate::services::llm::stream::stream_chat_no_done(
        &emitter,
        session_id,
        "request-xai-responses-runtime",
        0,
        1,
        "xai",
        FastModeRequest::Unsupported,
        RequestPurpose::ManualChat,
        "grok-4.6",
        &messages,
        &[],
        true,
        Some("high"),
        &previews,
        tokio_util::sync::CancellationToken::new(),
        false,
        None,
        None,
        None,
    )
    .await
    .expect("xAI Responses stream completes");

    let payloads = scenario.payloads();
    assert_eq!(payloads.len(), 1);
    assert_eq!(payloads[0]["reasoning"]["effort"], "high");
    assert!(payloads[0].get("messages").is_none());
}
