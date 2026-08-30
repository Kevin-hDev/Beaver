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

#[tokio::test]
async fn enabled_under_64k_candidate_uses_the_real_8192_window() {
    let mut session = super::snapshot_tests::session();
    session.messages = (0..8)
        .flat_map(|index| {
            let turn = uuid::Uuid::new_v4().to_string();
            [
                super::checkpoint_messages_tests::message(
                    &turn,
                    "user",
                    format!("user-{index} {}", "u".repeat(2_000)),
                ),
                super::checkpoint_messages_tests::message(
                    &turn,
                    "assistant",
                    format!("assistant-{index} {}", "a".repeat(2_000)),
                ),
            ]
        })
        .chain(std::iter::once(super::checkpoint_messages_tests::message(
            &uuid::Uuid::new_v4().to_string(),
            "user",
            "current work",
        )))
        .collect();
    let mut document = super::profile_store_document::CompressionProfileDocument::default();
    document.profiles[0].allow_under_64k = true;
    let profile = super::profile_resolve::resolve_from_document(None, &document).unwrap();
    let capabilities = super::session_capabilities::SessionCompressionCapabilities::from_runtime(
        false,
        &[],
        false,
        false,
        false,
    )
    .unwrap();
    let tiny = super::snapshot::CompressionSnapshot::capture(
        &session,
        profile,
        8_192,
        capabilities,
        super::profile_types::CompressionTrigger::Explicit,
    )
    .unwrap()
    .with_runtime_context(Vec::new(), Vec::new(), 7_500)
    .unwrap();

    let candidate = checkpoint_candidate::build(&tiny, Some(&summary()), &[])
        .await
        .expect("8K candidate");
    assert!(
        candidate.after_tokens <= 4_096,
        "{}",
        candidate.after_tokens
    );
}

#[tokio::test]
async fn disabled_tool_category_excludes_closed_tool_chains() {
    let mut session = super::snapshot_tests::session();
    let turn = uuid::Uuid::new_v4().to_string();
    let call_id = uuid::Uuid::new_v4().to_string();
    let mut assistant = super::checkpoint_messages_tests::message(&turn, "assistant", "");
    assistant.tool_calls = Some(vec![ToolCallRequest {
        id: call_id.clone(),
        extra_content: None,
        function: ToolCallRequestFunction {
            name: "web_search".into(),
            arguments: serde_json::json!({"q": "beaver"}),
        },
    }]);
    let mut tool = super::checkpoint_messages_tests::message(&turn, "tool", "tool evidence");
    tool.tool_name = Some("web_search".into());
    tool.tool_call_id = Some(call_id);
    session.messages = vec![
        super::checkpoint_messages_tests::message(&turn, "user", "question"),
        assistant,
        tool,
        super::checkpoint_messages_tests::message(&turn, "assistant", "final answer"),
        super::checkpoint_messages_tests::message(
            &uuid::Uuid::new_v4().to_string(),
            "user",
            "current work",
        ),
    ];
    let mut document = super::profile_store_document::CompressionProfileDocument::default();
    document.profiles[0].compact.tools.enabled = false;
    let profile = super::profile_resolve::resolve_from_document(None, &document).unwrap();
    let capabilities = super::session_capabilities::SessionCompressionCapabilities::from_runtime(
        false,
        &["web_search".into()],
        false,
        false,
        false,
    )
    .unwrap();
    let snapshot = super::snapshot::CompressionSnapshot::capture(
        &session,
        profile,
        96_000,
        capabilities,
        super::profile_types::CompressionTrigger::Explicit,
    )
    .unwrap()
    .with_runtime_context(Vec::new(), Vec::new(), 80_000)
    .unwrap();

    let candidate = checkpoint_candidate::build(&snapshot, Some(&summary()), &[])
        .await
        .unwrap();
    assert!(!candidate
        .persisted_messages
        .iter()
        .any(|message| message.role == "tool" || message.tool_calls.is_some()));
}

#[tokio::test]
async fn durable_checkpoint_metadata_is_not_projected_to_the_provider() {
    let session = stored_session().await;
    let captured = snapshot(&session);
    let candidate = checkpoint_candidate::build(&captured, Some(&summary()), &[])
        .await
        .unwrap();
    let persisted = candidate
        .persisted_messages
        .iter()
        .find(|message| message.message_kind == Some(AgentMessageKind::CompressionCheckpoint))
        .unwrap();
    let body: serde_json::Value = serde_json::from_str(&persisted.content).unwrap();
    let metadata = &body["metadata"];

    assert_eq!(metadata["profile_id"], "beaver");
    assert_eq!(metadata["profile_revision"], 1);
    assert_eq!(metadata["before_tokens"], captured.before_tokens);
    assert_eq!(metadata["after_tokens"], candidate.after_tokens);
    assert_eq!(metadata["trigger"], "explicit");
    assert!(candidate
        .runtime_messages
        .iter()
        .all(|message| !message.content.contains("\"metadata\"")));
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}
