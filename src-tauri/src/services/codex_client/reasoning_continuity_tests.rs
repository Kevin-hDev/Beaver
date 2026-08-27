use super::request::build_codex_request_with_continuity;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::llm::fast_mode::FastModeRequest;
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, CredentialScope, ReasoningModeId, ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};

fn target(scope: &str) -> ContinuationTarget {
    ContinuationTarget::FixtureCandidate(ReplayTarget {
        route_id: RouteId::CodexOauth,
        model_id: "gpt-5.6-luna".into(),
        credential_scope: CredentialScope::authenticated(scope).unwrap(),
        reasoning_mode: ReasoningModeId::Medium,
        continuation_use: ContinuationUse::UserContinuation,
    })
}

fn assistant(target: &ContinuationTarget) -> ChatMessage {
    ChatMessage::assistant(
        "visible".into(),
        None,
        Some(ReasoningEnvelope::new(
            crate::services::reasoning_continuity::contract::ContractId::CodexResponsesV1,
            ReasoningSource::from_target(target.replay().unwrap()),
            CompletionState::Complete,
            ContinuationState::ResponsesLocal {
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

#[test]
fn codex_payload_replays_items_before_current_user_without_legacy_tool_storage() {
    let target = target("codex-scope");
    let request = build_codex_request_with_continuity(
        "gpt-5.6-luna",
        &[assistant(&target), ChatMessage::user("continue".into())],
        &[],
        Some("medium"),
        Some("session"),
        FastModeRequest::Standard,
        Some(&target),
    )
    .unwrap();

    assert_eq!(request.input[0]["type"], "reasoning");
    assert_eq!(request.input[1]["type"], "message");
    assert_eq!(request.input[2]["role"], "user");
}

#[test]
fn codex_payload_rejects_scope_mismatch_without_falling_back_to_visible_text() {
    let captured = target("codex-scope");
    let request = build_codex_request_with_continuity(
        "gpt-5.6-luna",
        &[assistant(&captured), ChatMessage::user("continue".into())],
        &[],
        Some("medium"),
        Some("session"),
        FastModeRequest::Standard,
        Some(&target("new-login-scope")),
    );

    assert!(matches!(request, Err(error) if error == "reasoning_continuity_invalid"));
}
