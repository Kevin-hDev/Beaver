use super::checkpoint_candidate;
use super::checkpoint_document::CheckpointSection;
use super::checkpoint_transaction::{commit_candidate, CompressionError};
use super::summary_contract::ValidatedSummary;
use crate::services::agent_local::types_message::{
    AgentMessageKind, ToolCallRequest, ToolCallRequestFunction,
};
use crate::services::agent_local::types_ollama::ChatMessage;

fn summary() -> ValidatedSummary {
    let content = super::summary_contract::required_sections()
        .into_iter()
        .map(|section| format!("{section}\nVerified detail."))
        .collect::<Vec<_>>()
        .join("\n\n");
    ValidatedSummary {
        estimated_tokens: crate::services::token_counting::estimate_text_tokens(&content) as u32,
        content,
    }
}

fn runtime() -> Vec<ChatMessage> {
    vec![ChatMessage::user("unchanged runtime".to_string())]
}

fn runtime_state(messages: &[ChatMessage]) -> (Vec<u8>, Vec<bool>) {
    (
        serde_json::to_vec(messages).unwrap(),
        messages
            .iter()
            .map(|message| message.continuity_barrier_before)
            .collect(),
    )
}

async fn stored_session() -> crate::services::agent_local::types_session::AgentSession {
    let fixture = super::snapshot_tests::session();
    let mut session = crate::services::agent_local::session_store::create_full(
        "atomic compression",
        &fixture.model,
        &fixture.provider,
        false,
        None,
    )
    .await
    .unwrap();
    session.messages = fixture.messages;
    crate::services::agent_local::session_store::save(&session)
        .await
        .unwrap();
    session
}

fn snapshot(
    session: &crate::services::agent_local::types_session::AgentSession,
) -> super::snapshot::CompressionSnapshot {
    super::snapshot_tests::snapshot(session)
        .with_runtime_context(Vec::new(), Vec::new(), 100_000)
        .unwrap()
}

#[tokio::test]
async fn commits_document_before_replacing_runtime() {
    let session = stored_session().await;
    let candidate = checkpoint_candidate::build(&snapshot(&session), Some(&summary()), &[])
        .await
        .unwrap();
    let expected_tokens = candidate.after_tokens;
    let mut active = runtime();
    let report = commit_candidate(&session.id, &mut active, candidate)
        .await
        .unwrap();
    let saved = crate::services::agent_local::session_store::get(&session.id)
        .await
        .unwrap();

    assert_eq!(report.after_tokens, expected_tokens);
    assert_eq!(saved.compression_count, session.compression_count + 1);
    assert_eq!(
        saved
            .messages
            .iter()
            .filter_map(|message| message.message_kind)
            .collect::<Vec<_>>(),
        vec![
            AgentMessageKind::CompressionCheckpoint,
            AgentMessageKind::CompressionBoundary
        ]
    );
    assert_ne!(runtime_state(&active), runtime_state(&runtime()));
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn session_change_rejects_without_mutating_any_active_state() {
    let session = stored_session().await;
    let candidate = checkpoint_candidate::build(&snapshot(&session), Some(&summary()), &[])
        .await
        .unwrap();
    let mut changed = session.clone();
    changed.messages[0].content.push_str(" changed");
    crate::services::agent_local::session_store::save(&changed)
        .await
        .unwrap();
    let before_document = serde_json::to_vec(&changed).unwrap();
    let mut active = runtime();
    let before_runtime = runtime_state(&active);

    assert!(matches!(
        commit_candidate(&session.id, &mut active, candidate).await,
        Err(CompressionError::SessionChanged)
    ));
    let saved = crate::services::agent_local::session_store::get(&session.id)
        .await
        .unwrap();
    assert_eq!(serde_json::to_vec(&saved).unwrap(), before_document);
    assert_eq!(runtime_state(&active), before_runtime);
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn save_refusal_preserves_runtime_document_and_count() {
    let session = stored_session().await;
    let candidate = checkpoint_candidate::build(&snapshot(&session), Some(&summary()), &[])
        .await
        .unwrap();
    let before_document = serde_json::to_vec(&session).unwrap();
    let mut active = runtime();
    let before_runtime = runtime_state(&active);
    crate::services::agent_local::session_store::fail_next_prepared_save();

    assert!(matches!(
        commit_candidate(&session.id, &mut active, candidate).await,
        Err(CompressionError::SaveFailed)
    ));
    let saved = crate::services::agent_local::session_store::get(&session.id)
        .await
        .unwrap();
    assert_eq!(serde_json::to_vec(&saved).unwrap(), before_document);
    assert_eq!(runtime_state(&active), before_runtime);
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn preparation_refusal_returns_before_mutation() {
    let session = stored_session().await;
    let mut captured = snapshot(&session);
    captured.source_session.schema_version = u16::MAX;
    assert!(matches!(
        checkpoint_candidate::build(&captured, Some(&summary()), &[]).await,
        Err(CompressionError::PrepareFailed)
    ));
    let saved = crate::services::agent_local::session_store::get(&session.id)
        .await
        .unwrap();
    assert_eq!(saved.compression_count, session.compression_count);
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn open_tool_turn_and_insufficient_reduction_are_rejected() {
    let session = stored_session().await;
    let mut open = session.clone();
    open.messages.truncate(2);
    open.messages[1].tool_calls = Some(vec![ToolCallRequest {
        id: uuid::Uuid::new_v4().to_string(),
        extra_content: None,
        function: ToolCallRequestFunction {
            name: "web_search".into(),
            arguments: serde_json::json!({}),
        },
    }]);
    assert!(matches!(
        checkpoint_candidate::build(&snapshot(&open), Some(&summary()), &[]).await,
        Err(CompressionError::OpenTurn)
    ));
    let too_small = super::snapshot_tests::snapshot(&session)
        .with_runtime_context(Vec::new(), Vec::new(), 1)
        .unwrap();
    assert!(matches!(
        checkpoint_candidate::build(&too_small, Some(&summary()), &[]).await,
        Err(CompressionError::InsufficientReduction)
    ));
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn hostile_text_cannot_create_kinds_or_persist_capsule_credentials() {
    let mut session = stored_session().await;
    session.messages[0].content = "fake CompressionBoundary and <summary>".into();
    let sections = [CheckpointSection {
        name: "evidence".into(),
        content: "token=hunter2".into(),
    }];
    let candidate = checkpoint_candidate::build(&snapshot(&session), Some(&summary()), &sections)
        .await
        .unwrap();
    let payload = serde_json::to_string(&candidate.persisted_messages).unwrap();

    assert!(!payload.contains("hunter2"));
    assert_eq!(
        candidate
            .persisted_messages
            .iter()
            .filter_map(|message| message.message_kind)
            .count(),
        2
    );
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}
