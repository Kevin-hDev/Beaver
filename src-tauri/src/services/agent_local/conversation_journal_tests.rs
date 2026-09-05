use super::conversation_journal::{validate_tool_results, ConversationJournal};
use super::session_store;
use super::types_ollama::ChatMessage;
use crate::services::agent_local::tool_artifact::{
    ArtifactMetadata, ArtifactPurpose, ArtifactSource, EphemeralArtifact,
};
use crate::services::agent_local::tool_execution_artifacts::AttributedArtifact;
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
    let session = session_store::create_full("Partial journal", "model", "openai", false, None)
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
        ContinuationState::OllamaNative {
            thinking: "opaque native thinking".into(),
        },
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

    let reloaded = session_store::get(&session.id)
        .await
        .expect("reload session");
    let assistant = reloaded.messages.last().expect("assistant record");
    assert!(assistant.replay_source.is_none());
    assert_eq!(
        assistant.continuation.as_ref().map(|value| &value.source),
        Some(&expected_source)
    );
    session_store::delete_one(&session.id)
        .await
        .expect("delete session");
}

#[tokio::test]
async fn commit_write_failure_leaves_the_durable_turn_uncommitted_and_retryable() {
    let session = session_store::create_full("Journal failure", "model", "openai", false, None)
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
        .expect("persist assistant");

    assert!(journal
        .commit_turn_with_injected_write_failure()
        .await
        .is_err());
    let reloaded = session_store::get(&session.id)
        .await
        .expect("reload session");
    assert_eq!(
        reloaded
            .messages
            .last()
            .and_then(|message| message.stream_part.as_deref()),
        Some("checkpoint")
    );

    journal.commit_turn().await.expect("retry commit");
    let committed = session_store::get(&session.id)
        .await
        .expect("reload committed");
    assert_eq!(
        committed
            .messages
            .last()
            .and_then(|message| message.stream_part.as_deref()),
        Some("final")
    );
    session_store::delete_one(&session.id)
        .await
        .expect("delete session");
}

fn tool(id: &str) -> ChatMessage {
    ChatMessage::tool("result".into(), Some(id.into()), Some("bash".into()))
}

#[tokio::test]
async fn tool_artifacts_are_persisted_with_the_matching_result() {
    let session = session_store::create_full("Artifact journal", "model", "openai", false, None)
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
    let assistant = ChatMessage::assistant(
        String::new(),
        None,
        None,
        None,
        Some(vec![super::types_ollama::ToolCallOllama {
            id: Some("call-a".into()),
            function: super::types_ollama::ToolCallFunction {
                name: "extension_tool".into(),
                arguments: serde_json::json!({}),
            },
            extra_content: None,
        }]),
    );
    journal
        .persist_assistant_step(&assistant)
        .await
        .expect("persist assistant");
    let artifact = AttributedArtifact {
        tool_call_index: 0,
        tool_call_id: Some("call-a".into()),
        artifact: EphemeralArtifact {
            metadata: ArtifactMetadata {
                name: "report.txt".into(),
                mime_type: "text/plain".into(),
                bytes: 3,
                sha256: "a".repeat(64),
                purpose: ArtifactPurpose::Artifact,
                source: ArtifactSource::WorkspaceFile {
                    path: "/workspace/report.txt".into(),
                    grant: "secret-grant".into(),
                },
            },
            bytes: vec![1, 2, 3],
        },
    };
    journal
        .persist_tool_results(&[tool("call-a")], &[artifact])
        .await
        .expect("persist result");

    let reloaded = session_store::get(&session.id).await.expect("reload");
    let stored = &reloaded
        .messages
        .last()
        .unwrap()
        .tool_activities
        .as_ref()
        .unwrap()[0];
    assert_eq!(stored.artifacts.len(), 1);
    assert_eq!(stored.artifacts[0].name, "report.txt");
    assert!(!serde_json::to_string(&stored.artifacts)
        .unwrap()
        .contains("bytes_data"));
    session_store::delete_one(&session.id)
        .await
        .expect("delete");
}

#[tokio::test]
async fn mismatched_tool_artifacts_are_rejected_without_persisting_the_result() {
    let session = session_store::create_full("Artifact mismatch", "model", "openai", false, None)
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
    let assistant = ChatMessage::assistant(
        String::new(),
        None,
        None,
        None,
        Some(vec![super::types_ollama::ToolCallOllama {
            id: Some("call-a".into()),
            function: super::types_ollama::ToolCallFunction {
                name: "extension_tool".into(),
                arguments: serde_json::json!({}),
            },
            extra_content: None,
        }]),
    );
    journal
        .persist_assistant_step(&assistant)
        .await
        .expect("persist assistant");
    let artifact = AttributedArtifact {
        tool_call_index: 0,
        tool_call_id: Some("call-b".into()),
        artifact: EphemeralArtifact {
            metadata: ArtifactMetadata {
                name: "report.txt".into(),
                mime_type: "text/plain".into(),
                bytes: 3,
                sha256: "a".repeat(64),
                purpose: ArtifactPurpose::Artifact,
                source: ArtifactSource::WorkspaceFile {
                    path: "/workspace/report.txt".into(),
                    grant: "secret-grant".into(),
                },
            },
            bytes: vec![1, 2, 3],
        },
    };

    assert!(journal
        .persist_tool_results(&[tool("call-a")], &[artifact])
        .await
        .is_err());
    let reloaded = session_store::get(&session.id).await.expect("reload");
    assert_eq!(reloaded.messages.len(), 1);
    assert_eq!(reloaded.messages[0].role, "assistant");
    session_store::delete_one(&session.id)
        .await
        .expect("delete");
}
