use super::conversation_history_validation;
use super::conversation_interrupted_tail::close_recoverable;
use super::conversation_history_tests::support::{
    cleanup, complete_turn, message, resolved, tool_result,
};
use super::types_message::{ToolCallRequest, ToolCallRequestFunction};
use super::types_session::AgentSession;
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, NonReplayTarget, ReasoningModeId, RouteId,
};

async fn recoverable_tail() -> AgentSession {
    let mut session = super::conversation_history_tests::support::create_session().await;
    let turn_id = "turn-interrupted";
    let mut assistant = message("assistant-tools", turn_id, "assistant", "");
    assistant.tool_calls = Some(vec![ToolCallRequest {
        id: "call-read".into(),
        extra_content: None,
        function: ToolCallRequestFunction {
            name: "read_file".into(),
            arguments: serde_json::json!({"path": "logs/beaver.log"}),
        },
    }]);
    session.messages = vec![
        message("user-interrupted", turn_id, "user", "inspect logs"),
        assistant,
        tool_result(
            "result-read",
            turn_id,
            "call-read",
            "read_file",
            "bounded result",
        ),
    ];
    session
}

#[tokio::test]
async fn closes_only_a_tail_with_all_tool_results() {
    let mut session = recoverable_tail().await;
    assert!(conversation_history_validation::validate(&session.messages).is_err());

    assert!(close_recoverable(&mut session).expect("recoverable tail"));

    let terminal = session.messages.last().expect("terminal marker");
    assert_eq!(terminal.role, "assistant");
    assert_eq!(terminal.turn_id, "turn-interrupted");
    assert!(terminal.content.is_empty());
    assert!(terminal.tool_calls.is_none());
    assert!(terminal.continuation.is_none());
    assert_eq!(uuid::Uuid::parse_str(&terminal.id).unwrap().get_version_num(), 4);
    conversation_history_validation::validate(&session.messages).expect("strictly closed");

    super::conversation_history_tests::support::cleanup(&session.id).await;
}

#[tokio::test]
async fn closing_is_idempotent() {
    let mut session = recoverable_tail().await;
    assert!(close_recoverable(&mut session).unwrap());
    let once = session.messages.len();

    assert!(!close_recoverable(&mut session).unwrap());
    assert_eq!(session.messages.len(), once);

    super::conversation_history_tests::support::cleanup(&session.id).await;
}

#[tokio::test]
async fn missing_tool_result_is_never_invented() {
    let mut session = recoverable_tail().await;
    session.messages.pop();
    let before = serde_json::to_value(&session.messages).unwrap();

    assert!(close_recoverable(&mut session).is_err());
    assert_eq!(serde_json::to_value(&session.messages).unwrap(), before);

    super::conversation_history_tests::support::cleanup(&session.id).await;
}

#[tokio::test]
async fn copies_only_a_valid_stream_run_as_final() {
    let mut session = recoverable_tail().await;
    let run_id = uuid::Uuid::new_v4().to_string();
    let result = session.messages.last_mut().unwrap();
    result.stream_run_id = Some(run_id.clone());
    result.stream_part = Some("checkpoint".into());

    close_recoverable(&mut session).unwrap();

    let marker = session.messages.last().unwrap();
    assert_eq!(marker.stream_run_id.as_deref(), Some(run_id.as_str()));
    assert_eq!(marker.stream_part.as_deref(), Some("final"));
    super::conversation_history_tests::support::cleanup(&session.id).await;
}

#[tokio::test]
async fn invalid_history_stays_closed() {
    let mut session = recoverable_tail().await;
    session.messages.last_mut().unwrap().role = "unknown".into();

    assert!(close_recoverable(&mut session).is_err());

    super::conversation_history_tests::support::cleanup(&session.id).await;
}

#[tokio::test]
async fn conversation_admission_closes_the_real_interrupted_shape_atomically() {
    let mut session = recoverable_tail().await;
    let tail = std::mem::take(&mut session.messages);
    session.messages = complete_turn("older", "previous answer", None);
    session.messages.extend(tail);
    super::session_store::save(&session).await.unwrap();
    let previous = serde_json::to_value(&session.messages).unwrap();

    let admitted = super::conversation_admission::new_turn_for_continuation(
        &session.id,
        resolved("continue after cancellation"),
        forbidden(RouteId::Ollama),
    )
    .await
    .expect("admit after completed tool result");

    let loaded = super::session_store::get(&session.id).await.unwrap();
    let previous_len = previous.as_array().unwrap().len();
    assert_eq!(
        serde_json::to_value(&loaded.messages[..previous_len]).unwrap(),
        previous
    );
    assert_eq!(loaded.messages[previous_len].role, "assistant");
    assert!(loaded.messages[previous_len].content.is_empty());
    assert_eq!(loaded.messages[previous_len + 1].id, admitted.user_message_id);
    conversation_history_validation::validate(&loaded.messages).unwrap();
    cleanup(&session.id).await;
}

#[tokio::test]
async fn conversation_admission_failure_does_not_persist_the_repair() {
    let session = recoverable_tail().await;
    super::session_store::save(&session).await.unwrap();
    let before = std::fs::read(session_path(&session.id)).unwrap();
    let repair_seen = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let writer_seen = std::sync::Arc::clone(&repair_seen);

    let result = super::conversation_admission::new_turn_with_writer(
        &session.id,
        resolved("retry"),
        super::conversation_history_tests::support::target("model-a"),
        move |candidate| async move {
            writer_seen.store(
                candidate.messages.iter().any(|message| {
                    message.role == "assistant"
                        && message.content.is_empty()
                        && message.tool_calls.is_none()
                        && message.turn_id == "turn-interrupted"
                }),
                std::sync::atomic::Ordering::SeqCst,
            );
            Err("simulated write failure".into())
        },
    )
    .await;

    assert!(result.is_err());
    assert!(repair_seen.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(std::fs::read(session_path(&session.id)).unwrap(), before);
    cleanup(&session.id).await;
}

#[tokio::test]
async fn conversation_admission_is_shared_by_cloud_and_ollama() {
    for (provider, route) in [("openai", RouteId::OpenAi), ("ollama", RouteId::Ollama)] {
        let mut session = recoverable_tail().await;
        session.provider = provider.into();
        super::session_store::save(&session).await.unwrap();

        super::conversation_admission::new_turn_for_continuation(
            &session.id,
            resolved("continue"),
            forbidden(route),
        )
        .await
        .expect("shared admission accepts repaired history");

        let loaded = super::session_store::get(&session.id).await.unwrap();
        assert_eq!(
            loaded
                .messages
                .iter()
                .filter(|message| {
                    message.role == "assistant"
                        && message.content.is_empty()
                        && message.tool_calls.is_none()
                        && message.turn_id == "turn-interrupted"
                })
                .count(),
            1
        );
        cleanup(&session.id).await;
    }
}

#[tokio::test]
async fn conversation_admission_concurrency_adds_only_one_terminal_marker() {
    let session = recoverable_tail().await;
    super::session_store::save(&session).await.unwrap();
    let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(2));
    let first_id = session.id.clone();
    let second_id = session.id.clone();
    let first_barrier = std::sync::Arc::clone(&barrier);
    let first = tokio::spawn(async move {
        super::conversation_admission::new_turn_with_after_load(
            &first_id,
            resolved("first"),
            super::conversation_history_tests::support::target("model-a"),
            move || async move {
                first_barrier.wait().await;
            },
        )
        .await
    });
    barrier.wait().await;
    let second = tokio::spawn(async move {
        super::conversation_admission::new_turn_for_continuation(
            &second_id,
            resolved("second"),
            forbidden(RouteId::Ollama),
        )
        .await
    });

    first.await.unwrap().expect("first admission");
    assert!(second.await.unwrap().is_err());
    let loaded = super::session_store::get(&session.id).await.unwrap();
    assert_eq!(
        loaded
            .messages
            .iter()
            .filter(|message| {
                message.role == "assistant"
                    && message.content.is_empty()
                    && message.tool_calls.is_none()
                    && message.turn_id == "turn-interrupted"
            })
            .count(),
        1
    );
    cleanup(&session.id).await;
}

fn forbidden(route_id: RouteId) -> ContinuationTarget {
    ContinuationTarget::Forbidden(NonReplayTarget {
        route_id,
        model_id: "model-a".into(),
        reasoning_mode: ReasoningModeId::Auto,
    })
}

fn session_path(id: &str) -> std::path::PathBuf {
    crate::services::paths::data_dir()
        .join("agent-sessions")
        .join(format!("{id}.json"))
}
