use uuid::Uuid;

use super::super::{conversation_admission, conversation_history};
use super::support::{cleanup, complete_turn, create_session, envelope, message, multi_tool_turn, target, ERROR};
use crate::models::agent_session_contract::EditUserMessageInput;
use crate::models::agent_turn_contract::ResumeTurnInput;
use crate::services::reasoning_continuity::contract::RouteId;
use crate::services::reasoning_continuity::envelope::{ContinuationState, ReasoningEnvelope};

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
async fn resume_refuses_context_that_cannot_be_reconstructed_exactly() {
    let mut session = create_session().await;
    let mut contextual = message("context-user", "context-turn", "user", "question");
    contextual.skill_names = Some(vec!["Skill local".into()]);
    session.messages.push(contextual.clone());
    super::super::session_store::save(&session)
        .await
        .expect("seed contextual terminal");

    let error = conversation_admission::resume(
        &session.id,
        ResumeTurnInput {
            message_id: contextual.id,
        },
        target("model-a"),
    )
    .await
    .expect_err("contextual resume must fail closed");

    assert_eq!(error.to_string(), ERROR);
    cleanup(&session.id).await;
}

fn ollama_thinking(envelope: &ReasoningEnvelope) -> Option<&str> {
    match &envelope.continuation {
        ContinuationState::OllamaNative { thinking } => Some(thinking),
        _ => None,
    }
}
