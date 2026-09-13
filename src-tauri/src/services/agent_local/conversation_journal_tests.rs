use super::context_usage_record::{
    ContextCountCoverage, ContextCountSource, ContextMeasurementSnapshot, ContextOutputSnapshot,
    ContextPreparationSnapshot, ContextPreparationState, ContextRequestIdentity, ContextTokenCount,
};
use super::conversation_history_tests::support::message;
use super::conversation_journal::{validate_tool_results, ConversationJournal};
use super::session_store;
use super::stream_recovery_log::StreamRecoveryLog;
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
use tokio_util::sync::CancellationToken;

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

#[tokio::test]
async fn conversation_journal_stages_exact_messages_and_turn_before_session_writes() {
    let session = session_store::create_full("Recovery journal", "model", "openai", false, None)
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
    let (log, owner) =
        StreamRecoveryLog::create(journal.recovery_header(), CancellationToken::new())
            .await
            .expect("create recovery log");
    let path = log.path();
    journal.attach_recovery(log.clone(), owner);

    journal
        .persist_assistant_step(&ChatMessage::assistant(
            "durable assistant".into(),
            None,
            None,
            None,
            None,
        ))
        .await
        .expect("persist assistant");
    let records = recovery_records(&path);
    assert_eq!(records.len(), 1, "successful save clears pending batch");

    assert!(journal
        .commit_turn_with_injected_write_failure()
        .await
        .is_err());
    assert!(matches!(
        recovery_records(&path).last(),
        Some(super::stream_recovery_record::StreamRecoveryRecord::TurnReady { .. })
    ));

    journal.commit_turn().await.expect("retry commit");
    assert!(
        !path.exists(),
        "journal is removed only after durable commit"
    );
    session_store::delete_one(&session.id)
        .await
        .expect("delete session");
}

fn recovery_records(
    path: &std::path::Path,
) -> Vec<super::stream_recovery_record::StreamRecoveryRecord> {
    let mut records = Vec::new();
    super::stream_recovery_store::visit_records(path, |record| {
        records.push(record);
        Ok(())
    })
    .expect("read recovery records");
    records
}

fn tool(id: &str) -> ChatMessage {
    ChatMessage::tool("result".into(), Some(id.into()), Some("bash".into()))
}

#[tokio::test]
async fn superseded_run_cannot_append_a_late_tool_result() {
    let mut session =
        session_store::create_full("Superseded journal", "model", "openai", false, None)
            .await
            .expect("create session");
    let old_turn = uuid::Uuid::new_v4().to_string();
    let old_user = uuid::Uuid::new_v4().to_string();
    let old_assistant = uuid::Uuid::new_v4().to_string();
    let old_request = uuid::Uuid::new_v4().to_string();
    session
        .messages
        .push(message(&old_user, &old_turn, "user", "run tool"));
    session_store::save(&session)
        .await
        .expect("persist old user");
    let mut journal = ConversationJournal::new(
        session.id.clone(),
        old_turn.clone(),
        old_user,
        old_assistant,
        old_request,
    )
    .expect("create old journal");
    journal
        .persist_assistant_step(&ChatMessage::assistant(
            String::new(),
            None,
            None,
            None,
            Some(vec![super::types_ollama::ToolCallOllama {
                id: Some("call-late".into()),
                function: super::types_ollama::ToolCallFunction {
                    name: "bash".into(),
                    arguments: serde_json::json!({"command": "sleep 1"}),
                },
                extra_content: None,
            }]),
        ))
        .await
        .expect("persist pending call");

    let mut recovered = session_store::get(&session.id).await.expect("reload");
    let current_request = uuid::Uuid::new_v4().to_string();
    super::conversation_interrupted_tail::close_recoverable(
        &mut recovered,
        super::conversation_interrupted_tail::RecoveryProof::AdmissionFallback {
            current_execution_id: &current_request,
        },
    )
    .expect("close orphan");
    let new_turn = uuid::Uuid::new_v4().to_string();
    recovered.messages.push(message(
        &uuid::Uuid::new_v4().to_string(),
        &new_turn,
        "user",
        "continue",
    ));
    session_store::save(&recovered)
        .await
        .expect("persist recovery");

    assert!(journal
        .persist_tool_results(&[tool("call-late")], &[])
        .await
        .is_err());
    let saved = session_store::get(&session.id).await.expect("reload final");
    assert_eq!(saved.messages.last().unwrap().turn_id, new_turn);
    assert_eq!(
        saved
            .messages
            .iter()
            .filter(|item| item.tool_call_id.as_deref() == Some("call-late"))
            .count(),
        1,
    );
    super::conversation_history_validation::validate(&saved.messages).expect("history stays valid");
    session_store::delete_one(&session.id)
        .await
        .expect("delete session");
}

#[tokio::test]
async fn live_subagent_instruction_does_not_supersede_its_owned_journal() {
    let parent = session_store::create_full("Parent", "model", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Child", "model", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    let execution = super::subagent_registry::register_execution(
        &parent.id,
        &child.id,
        tokio_util::sync::CancellationToken::new(),
    )
    .await
    .expect("register child");
    child.subagent_run_id = Some(execution.run_id.clone());
    let turn_id = uuid::Uuid::new_v4().to_string();
    let user_id = uuid::Uuid::new_v4().to_string();
    child
        .messages
        .push(message(&user_id, &turn_id, "user", "initial mission"));
    session_store::save(&child).await.expect("save child");
    let mut journal = ConversationJournal::new_for_subagent(
        child.id.clone(),
        turn_id,
        user_id,
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        execution.run_id,
        execution.execution_id,
    )
    .expect("create child journal");
    journal
        .persist_assistant_step(&ChatMessage::assistant(
            "first step".into(),
            None,
            None,
            None,
            None,
        ))
        .await
        .expect("persist first step");

    let mut queued = session_store::get(&child.id).await.expect("reload child");
    super::subagent_instruction_delivery::enqueue(&mut queued, "new instruction")
        .expect("queue instruction");
    session_store::save(&queued).await.expect("save queue");
    super::subagent_instruction_delivery::drain(&child.id, &mut Vec::new())
        .await
        .expect("drain instruction");

    journal
        .persist_assistant_step(&ChatMessage::assistant(
            "continued step".into(),
            None,
            None,
            None,
            None,
        ))
        .await
        .expect("owned journal remains writable");
    journal.commit_turn().await.expect("commit child turn");
    let saved = session_store::get(&child.id).await.expect("reload child");
    assert_eq!(saved.messages.last().unwrap().content, "continued step");

    super::subagent_registry::unregister(&child.id).await;
    session_store::delete_one(&child.id)
        .await
        .expect("delete child");
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
}

fn context_identity(
    journal: &ConversationJournal,
    turn: u32,
    attempt: u32,
) -> ContextRequestIdentity {
    journal.context_identity(turn, attempt, "openai", "gpt-5")
}

fn context_count(tokens: u32, source: ContextCountSource) -> ContextTokenCount {
    ContextTokenCount {
        tokens: Some(tokens),
        capacity_tokens: Some(tokens),
        source: Some(source),
        coverage: ContextCountCoverage::Complete,
    }
}

fn preparation(
    journal: &ConversationJournal,
    turn: u32,
    attempt: u32,
    tokens: u32,
) -> ContextPreparationSnapshot {
    ContextPreparationSnapshot {
        identity: context_identity(journal, turn, attempt),
        context_limit: Some(200_000),
        input: context_count(tokens, ContextCountSource::Heuristic),
        state: ContextPreparationState::InFlight,
        breakdown: None,
        transient_overhead_tokens: 0,
        updated_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn newer_request_rejects_every_late_context_update() {
    let session = session_store::create_full("Context order", "gpt-5", "openai", false, None)
        .await
        .expect("create session");
    let turn_id = uuid::Uuid::new_v4().to_string();
    let journal = |request_id| {
        ConversationJournal::new(
            session.id.clone(),
            turn_id.clone(),
            uuid::Uuid::new_v4().to_string(),
            uuid::Uuid::new_v4().to_string(),
            request_id,
        )
        .expect("create journal")
    };
    let first = journal(uuid::Uuid::new_v4().to_string());
    first.activate_context_request().await.unwrap();
    assert!(first
        .persist_context_preparation(preparation(&first, 8, 1, 120))
        .await
        .unwrap());

    let second = journal(uuid::Uuid::new_v4().to_string());
    second.activate_context_request().await.unwrap();
    assert!(second
        .persist_context_preparation(preparation(&second, 0, 1, 80))
        .await
        .unwrap());
    assert!(!first
        .persist_context_preparation(preparation(&first, 9, 1, 999))
        .await
        .unwrap());
    assert!(!first
        .persist_context_measurement(ContextMeasurementSnapshot {
            identity: context_identity(&first, 9, 1),
            context_limit: Some(200_000),
            input: context_count(999, ContextCountSource::Provider),
            updated_at: chrono::Utc::now(),
        })
        .await
        .unwrap());
    first
        .finish_context_request(ContextPreparationState::Interrupted)
        .await
        .unwrap();

    let saved = session_store::get(&session.id).await.unwrap();
    assert_eq!(
        saved.context_usage.active_request_id.as_deref(),
        Some(context_identity(&second, 0, 1).request_id.as_str())
    );
    assert_eq!(
        saved
            .context_usage
            .current_preparation
            .unwrap()
            .input
            .tokens,
        Some(80)
    );
    assert!(saved.context_usage.last_measurement.is_none());
    session_store::delete_one(&session.id).await.unwrap();
}

#[tokio::test]
async fn context_attempts_measurements_and_output_keep_distinct_lifetimes() {
    let session = session_store::create_full("Context lifetime", "gpt-5", "openai", false, None)
        .await
        .expect("create session");
    let journal = ConversationJournal::new(
        session.id.clone(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
    )
    .unwrap();
    journal.activate_context_request().await.unwrap();
    assert!(journal
        .persist_context_preparation(preparation(&journal, 0, 2, 120))
        .await
        .unwrap());
    assert!(!journal
        .persist_context_preparation(preparation(&journal, 0, 1, 999))
        .await
        .unwrap());
    assert!(journal
        .persist_context_measurement(ContextMeasurementSnapshot {
            identity: context_identity(&journal, 0, 2),
            context_limit: Some(200_000),
            input: context_count(100, ContextCountSource::Provider),
            updated_at: chrono::Utc::now(),
        })
        .await
        .unwrap());
    assert!(journal
        .persist_context_output(ContextOutputSnapshot {
            identity: context_identity(&journal, 0, 2),
            output: context_count(50, ContextCountSource::Provider),
            updated_at: chrono::Utc::now(),
        })
        .await
        .unwrap());
    journal
        .finish_context_request(ContextPreparationState::Completed)
        .await
        .unwrap();

    let saved = session_store::get(&session.id).await.unwrap();
    assert_eq!(saved.context_usage.active_request_id, None);
    assert_eq!(
        saved.context_usage.current_preparation.unwrap().state,
        ContextPreparationState::Completed
    );
    assert_eq!(
        saved.context_usage.last_measurement.unwrap().input.tokens,
        Some(100)
    );
    assert_eq!(
        saved.context_usage.last_output.unwrap().output.tokens,
        Some(50)
    );
    session_store::delete_one(&session.id).await.unwrap();
}

#[tokio::test]
async fn stale_preparation_cannot_be_completed_after_invalidation() {
    let session = session_store::create_full("Stale context", "gpt-5", "openai", false, None)
        .await
        .unwrap();
    let journal = ConversationJournal::new(
        session.id.clone(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
    )
    .unwrap();
    journal.activate_context_request().await.unwrap();
    let identity = context_identity(&journal, 0, 1);
    assert!(journal
        .persist_context_preparation(preparation(&journal, 0, 1, 120))
        .await
        .unwrap());
    let mut invalidated = session_store::get(&session.id).await.unwrap();
    invalidated.context_usage.invalidate_preparation();
    session_store::save(&invalidated).await.unwrap();

    assert!(!journal.complete_context_attempt(&identity).await.unwrap());
    journal
        .finish_context_request(ContextPreparationState::Completed)
        .await
        .unwrap();

    let saved = session_store::get(&session.id).await.unwrap();
    assert_eq!(saved.context_usage.active_request_id, None);
    assert_eq!(
        saved.context_usage.current_preparation.unwrap().state,
        ContextPreparationState::Stale
    );
    session_store::delete_one(&session.id).await.unwrap();
}

#[tokio::test]
async fn full_session_reports_its_capacity_instead_of_a_generic_journal_failure() {
    let mut session = session_store::create_full("Full journal", "model", "ollama", false, None)
        .await
        .expect("create session");
    session.messages = (0..super::session_limits::MAX_MESSAGES_PER_SESSION)
        .map(|index| super::types_message::AgentMessage {
            id: uuid::Uuid::new_v4().to_string(),
            turn_id: uuid::Uuid::new_v4().to_string(),
            role: "user".into(),
            content: format!("message-{index}"),
            message_kind: None,
            thinking: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
            continuation: None,
            replay_source: None,
            tool_activities: None,
            segments: None,
            files: Vec::new(),
            timestamp: chrono::Utc::now(),
            tokens: 0,
            work_duration_ms: None,
            skill_names: None,
            skill_ids: None,
            stream_run_id: None,
            stream_part: None,
        })
        .collect();
    session_store::save(&session).await.expect("fill session");
    let mut journal = ConversationJournal::new(
        session.id.clone(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
    )
    .expect("create journal");

    let error = journal
        .persist_assistant_step(&ChatMessage::assistant(
            "overflow".into(),
            None,
            None,
            None,
            None,
        ))
        .await
        .expect_err("full session must reject the checkpoint");

    assert_eq!(error, "session_capacity_reached");
    session_store::delete_one(&session.id)
        .await
        .expect("delete session");
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
