use super::conversation_journal::{validate_tool_results, ConversationJournal};
use super::session_store;
use super::types_ollama::ChatMessage;
use crate::services::reasoning_continuity::contract::{
    ContinuationUse, CredentialScope, ReasoningModeId, ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};

#[test]
fn journal_rejects_missing_duplicate_and_reordered_tool_results() {
    let expected = vec!["call-a".to_string(), "call-b".to_string()];
    assert!(validate_tool_results(&[tool("call-a"), tool("call-b")], &expected).is_ok());
    assert!(validate_tool_results(&[tool("call-a")], &expected).is_err());
    assert!(validate_tool_results(&[tool("call-a"), tool("call-a")], &expected).is_err());
    assert!(validate_tool_results(&[tool("call-b"), tool("call-a")], &expected).is_err());
}

#[test]
fn journal_rejects_non_tool_messages_in_a_tool_result_batch() {
    let expected = vec!["call-a".to_string()];
    let mixed = vec![tool("call-a"), ChatMessage::user("follow-up".to_string())];

    assert!(validate_tool_results(&mixed, &expected).is_err());
}

#[tokio::test]
async fn partial_checkpoint_never_commits_a_turn_as_final() {
    let session = session_store::create_full("Partial journal", "model", "groq", false, None)
        .await
        .expect("create session");
    let mut journal = ConversationJournal::new(
        session.id.clone(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
    )
    .expect("create journal");
    journal
        .persist_assistant_step(&ChatMessage::assistant(
            "complete step".into(),
            None,
            None,
            None,
            None,
        ))
        .await
        .expect("persist complete step");
    journal
        .persist_partial(ChatMessage::assistant(
            "interrupted step".into(),
            None,
            None,
            None,
            None,
        ))
        .await
        .expect("persist partial step");

    assert!(journal.commit_turn().await.is_err());
    session_store::delete_one(&session.id)
        .await
        .expect("delete session");
}

#[tokio::test]
async fn assistant_envelope_persists_without_a_duplicate_replay_source() {
    let session = session_store::create_full("Journal Ollama", "qwen3.5:4b", "ollama", true, None)
        .await
        .expect("create session");
    let target = ReplayTarget {
        route_id: RouteId::Ollama,
        model_id: "qwen3.5:4b".into(),
        credential_scope: CredentialScope::local_uncredentialed(),
        reasoning_mode: ReasoningModeId::Auto,
        continuation_use: ContinuationUse::UserContinuation,
    };
    let envelope = ReasoningEnvelope::new(
        crate::services::reasoning_continuity::contract::ContractId::OllamaNativeV1,
        ReasoningSource::from_target(&target),
        CompletionState::Complete,
        ContinuationState::OllamaNative { thinking: "opaque native thinking".into() },
        Vec::new(),
    );
    let expected_source = envelope.source.clone();
    let mut journal = ConversationJournal::new(
        session.id.clone(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
    )
    .expect("create journal");
    journal
        .persist_assistant_step(&ChatMessage::assistant(
            "complete step".into(),
            Some("display thinking".into()),
            Some(envelope),
            None,
            None,
        ))
        .await
        .expect("persist assistant");
    journal.commit_turn().await.expect("commit turn");

    let reloaded = session_store::get(&session.id).await.expect("reload session");
    let assistant = reloaded.messages.last().expect("assistant record");
    assert!(assistant.replay_source.is_none());
    assert_eq!(assistant.continuation.as_ref().map(|value| &value.source), Some(&expected_source));
    session_store::delete_one(&session.id).await.expect("delete session");
}

fn tool(id: &str) -> ChatMessage { ChatMessage::tool("result".into(), Some(id.into()), Some("bash".into())) }
