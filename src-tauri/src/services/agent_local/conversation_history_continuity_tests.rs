use uuid::Uuid;

use super::super::{conversation_admission, conversation_history};
use super::support::{cleanup, complete_turn, create_session, envelope, message, multi_tool_turn, target, ERROR};
use crate::models::agent_session_contract::EditUserMessageInput;
use crate::models::agent_turn_contract::ResumeTurnInput;
use crate::services::reasoning_continuity::contract::RouteId;
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, ContractId, CredentialScope, ReasoningModeId, ReplayTarget,
};
use crate::services::reasoning_continuity::envelope::{
    CompletionState, ContinuationState, ReasoningEnvelope, ReasoningSource,
};
use crate::services::reasoning_continuity::tool_links::ToolLink;

#[tokio::test]
async fn display_thinking_never_becomes_continuation_and_opaque_is_exact() {
    let opaque = envelope(RouteId::Ollama, "model-a", "opaque exact");
    let expected = serde_json::to_vec(&opaque).unwrap();
    let mut session = create_session().await;
    session.messages = complete_turn("one", "answer", Some(opaque));
    session.messages[1].thinking = Some("display only".into());
    super::super::session_store::save(&session)
        .await
        .expect("seed");

    let history = conversation_history::load_for_target(&session.id, &target("model-a"))
        .await
        .expect("load");
    assert_eq!(
        serde_json::to_vec(history.messages[1].continuation.as_ref().unwrap()).unwrap(),
        expected
    );
    assert_eq!(
        history.messages[1].display_thinking.as_deref(),
        Some("display only")
    );

    session.messages[1].continuation = None;
    super::super::session_store::save(&session)
        .await
        .expect("remove opaque");
    let display_only = conversation_history::load_for_target(&session.id, &target("model-a"))
        .await
        .expect("load display only");
    assert!(display_only.messages[1].continuation.is_none());
    cleanup(&session.id).await;
}

#[cfg(debug_assertions)]
#[tokio::test]
async fn fixture_candidate_reloads_canonical_tool_links_after_restart() {
    let scope = CredentialScope::authenticated("fixture-scope").unwrap();
    let target = ReplayTarget {
        route_id: RouteId::OpenAi,
        model_id: "gpt-5.6-luna".into(),
        credential_scope: scope.clone(),
        reasoning_mode: ReasoningModeId::Medium,
        continuation_use: ContinuationUse::UserContinuation,
    };
    let source = ReasoningSource::from_target(&target);
    let mut session = super::super::session_store::create_full(
        "fixture restart",
        "gpt-5.6-luna",
        "openai",
        false,
        None,
    )
    .await
    .unwrap();
    session.reasoning_mode = Some("medium".into());
    session.thinking_enabled = true;
    let turn_id = "00000000-0000-4000-8000-000000000010";
    let mut user = message(
        "00000000-0000-4000-8000-000000000011",
        turn_id,
        "user",
        "use tool",
    );
    user.replay_source = Some(source.clone());
    let mut assistant = message(
        "00000000-0000-4000-8000-000000000012",
        turn_id,
        "assistant",
        "",
    );
    assistant.tool_calls = Some(vec![super::super::types_message::ToolCallRequest {
        id: "call-1".into(),
        extra_content: None,
        function: super::super::types_message::ToolCallRequestFunction {
            name: "fixture.write_note".into(),
            arguments: serde_json::json!({"value":"fixture"}),
        },
    }]);
    assistant.continuation = Some(ReasoningEnvelope::new(
        ContractId::OpenAiResponsesV1,
        source,
        CompletionState::Complete,
        ContinuationState::ResponsesLocal {
            items: vec![serde_json::json!({
                "type":"function_call", "call_id":"call-1", "name":"fixture_write_note"
            })],
        },
        vec![ToolLink {
            provider_call_id: "call-1".into(),
            tool_name: "fixture.write_note".into(),
        }],
    ));
    let tool = super::support::tool_result(
        "00000000-0000-4000-8000-000000000013",
        turn_id,
        "call-1",
        "fixture.write_note",
        "ok",
    );
    let final_assistant = message(
        "00000000-0000-4000-8000-000000000014",
        turn_id,
        "assistant",
        "done",
    );
    session.messages = vec![user, assistant, tool, final_assistant];
    super::super::session_store::save(&session).await.unwrap();

    let reloaded = super::super::session_store::get(&session.id).await.unwrap();
    let history = super::super::conversation_history_build::from_continuation(
        &reloaded,
        &ContinuationTarget::FixtureCandidate(target),
    )
    .expect("canonical fixture history after restart");
    let envelope = history.messages[1].continuation.as_ref().unwrap();
    assert_eq!(envelope.tool_links[0].tool_name, "fixture.write_note");
    let ContinuationState::ResponsesLocal { items } = &envelope.continuation else {
        panic!("responses continuation");
    };
    assert_eq!(items[0]["name"], "fixture_write_note");
    cleanup(&session.id).await;
}

#[tokio::test]
async fn provenance_barrier_exposes_only_a_new_compatible_suffix() {
    let mut session = create_session().await;
    session.messages = complete_turn(
        "old-a",
        "old A",
        Some(envelope(RouteId::Ollama, "model-a", "opaque-old-a")),
    );
    session.messages.extend(complete_turn(
        "middle-b",
        "middle B",
        Some(envelope(RouteId::Ollama, "model-b", "opaque-b")),
    ));
    session.messages.extend(complete_turn(
        "new-a",
        "new A",
        Some(envelope(RouteId::Ollama, "model-a", "opaque-new-a")),
    ));
    super::super::session_store::save(&session)
        .await
        .expect("seed barriers");

    let history = conversation_history::load_for_target(&session.id, &target("model-a"))
        .await
        .expect("load A suffix");

    assert_eq!(history.compatible_suffix_start, 4);
    assert!(history.messages[1].continuation.is_none());
    assert!(history.messages[3].continuation.is_none());
    assert_eq!(
        history.messages[5]
            .continuation
            .as_ref()
            .and_then(ollama_thinking),
        Some("opaque-new-a")
    );
    assert!(history.messages[4].continuity_barrier_before);
    cleanup(&session.id).await;
}

#[tokio::test]
async fn durable_turn_provenance_blocks_a_to_b_without_envelope_then_a_after_reload() {
    let old_a = envelope(RouteId::Ollama, "model-a", "opaque-old-a");
    let b_source = envelope(RouteId::Ollama, "model-b", "unused").source;
    let new_a = envelope(RouteId::Ollama, "model-a", "opaque-new-a");
    let mut session = create_session().await;
    session.messages = complete_turn("old-a-source", "old A", Some(old_a.clone()));
    session.messages.extend(complete_turn("middle-b-source", "middle B", None));
    session.messages.extend(complete_turn("new-a-source", "new A", Some(new_a.clone())));
    let mut value = serde_json::to_value(&session).unwrap();
    let messages = value["messages"].as_array_mut().unwrap();
    messages[0]["replay_source"] = serde_json::to_value(old_a.source).unwrap();
    messages[2]["replay_source"] = serde_json::to_value(b_source).unwrap();
    messages[4]["replay_source"] = serde_json::to_value(new_a.source).unwrap();
    crate::services::private_store::atomic_write_async(
        super::support::session_path(&session.id),
        serde_json::to_vec(&value).unwrap(),
    )
    .await
    .unwrap();

    let history = conversation_history::load_for_target(&session.id, &target("model-a"))
        .await
        .expect("reload durable turn provenance");

    assert_eq!(history.compatible_suffix_start, 4);
    assert!(history.messages[1].continuation.is_none());
    assert!(history.messages[4].continuity_barrier_before);
    assert_eq!(history.messages[5].continuation.as_ref().and_then(ollama_thinking), Some("opaque-new-a"));
    cleanup(&session.id).await;
}

#[tokio::test]
async fn resume_accepts_only_last_terminal_user_without_mutating_history() {
    let mut session = create_session().await;
    session.messages = complete_turn("old", "done", None);
    let terminal = message(
        "00000000-0000-4000-8000-000000000020",
        "terminal-turn",
        "user",
        "retry me",
    );
    session.messages.push(terminal.clone());
    super::super::session_store::save(&session)
        .await
        .expect("seed terminal user");

    let resumed = conversation_admission::resume(
        &session.id,
        ResumeTurnInput {
            message_id: terminal.id.clone(),
        },
        target("model-a"),
    )
    .await
    .expect("resume terminal user");

    assert_eq!(resumed.turn_id, terminal.turn_id);
    assert_eq!(resumed.user_message_id, terminal.id);
    assert_ne!(resumed.assistant_message_id, resumed.turn_id);
    assert_ne!(resumed.assistant_message_id, resumed.user_message_id);
    assert_eq!(
        Uuid::parse_str(&resumed.assistant_message_id)
            .unwrap()
            .get_version_num(),
        4
    );
    let unchanged = super::super::session_store::get(&session.id)
        .await
        .expect("reload unchanged");
    assert_eq!(unchanged.messages.len(), 3);

    for invalid in ["user-old", "assistant-old", "missing-message"] {
        let error = conversation_admission::resume(
            &session.id,
            ResumeTurnInput {
                message_id: invalid.into(),
            },
            target("model-a"),
        )
        .await
        .expect_err("non-terminal target rejected");
        assert_eq!(error.to_string(), ERROR);
    }
    cleanup(&session.id).await;
}

#[tokio::test]
async fn edit_truncates_turns_keeps_prior_opaque_and_makes_user_terminal() {
    let first_opaque = envelope(RouteId::Ollama, "model-a", "first exact");
    let expected = serde_json::to_vec(&first_opaque).unwrap();
    let mut session = create_session().await;
    session.messages = complete_turn("first", "first answer", Some(first_opaque));
    session.messages.extend(multi_tool_turn("second"));
    session
        .messages
        .extend(complete_turn("third", "third answer", None));
    let prior_turn = serde_json::to_vec(&session.messages[..2]).unwrap();
    let edited_id = session.messages[2].id.clone();
    let edited_turn = session.messages[2].turn_id.clone();
    super::super::session_store::save(&session)
        .await
        .expect("seed edit");

    let history = conversation_admission::edit_user_message(
        &session.id,
        EditUserMessageInput {
            message_id: edited_id.clone(),
            new_content: "edited durable".into(),
        },
        &target("model-a"),
    )
    .await
    .expect("edit by whole turns");

    let persisted = super::super::session_store::get(&session.id)
        .await
        .expect("reload edit");
    assert_eq!(persisted.messages.len(), 3);
    assert_eq!(persisted.messages[2].id, edited_id);
    assert_eq!(persisted.messages[2].turn_id, edited_turn);
    assert_eq!(persisted.messages[2].content, "edited durable");
    assert_eq!(serde_json::to_vec(&persisted.messages[..2]).unwrap(), prior_turn);
    assert_eq!(
        serde_json::to_vec(persisted.messages[1].continuation.as_ref().unwrap()).unwrap(),
        expected
    );
    assert_eq!(history.messages.last().unwrap().content, "edited durable");
    cleanup(&session.id).await;
}

#[tokio::test]
async fn failed_edit_write_is_generic_and_preserves_previous_bytes() {
    let mut session = create_session().await;
    session.messages = complete_turn("edit-fail", "answer", None);
    super::super::session_store::save(&session).await.unwrap();
    let path = super::support::session_path(&session.id);
    let before = std::fs::read(&path).unwrap();

    let error = conversation_admission::edit_user_message_with_writer(
        &session.id,
        EditUserMessageInput { message_id: "user-edit-fail".into(), new_content: "changed".into() },
        &target("model-a"),
        |_| async { Err("technical /private/path".to_string()) },
    )
    .await
    .expect_err("edit write fails");

    assert_eq!(error.to_string(), ERROR);
    assert_eq!(std::fs::read(path).unwrap(), before);
    cleanup(&session.id).await;
}

#[tokio::test]
async fn successful_edit_returns_the_exact_sanitized_document_history() {
    let mut session = create_session().await;
    session.messages = complete_turn("edit-sanitized", "answer", None);
    super::super::session_store::save(&session).await.unwrap();

    let history = conversation_admission::edit_user_message(
        &session.id,
        EditUserMessageInput {
            message_id: "user-edit-sanitized".into(),
            new_content: "use gsk_1234567890abcdefghijkl".into(),
        },
        &target("model-a"),
    )
    .await
    .expect("edit canonical prepared document");

    let persisted = super::super::session_store::get(&session.id).await.unwrap();
    assert_eq!(history.messages.last().unwrap().content, persisted.messages[0].content);
    assert!(!persisted.messages[0].content.contains("gsk_1234567890abcdefghijkl"));
    cleanup(&session.id).await;
}

#[tokio::test]
async fn resume_keeps_legacy_skill_names_without_inventing_private_context() {
    let mut session = create_session().await;
    let mut contextual = message("context-user", "context-turn", "user", "question");
    contextual.skill_names = Some(vec!["Skill local".into()]);
    session.messages.push(contextual.clone());
    super::super::session_store::save(&session)
        .await
        .expect("seed contextual terminal");

    let admitted = conversation_admission::resume(
        &session.id,
        ResumeTurnInput {
            message_id: contextual.id,
        },
        target("model-a"),
    )
    .await
    .expect("legacy names remain readable without private skill ids");

    assert_eq!(admitted.history.messages.last().unwrap().content, "question");
    cleanup(&session.id).await;
}

fn ollama_thinking(envelope: &ReasoningEnvelope) -> Option<&str> {
    match &envelope.continuation {
        ContinuationState::OllamaNative { thinking } => Some(thinking),
        _ => None,
    }
}
