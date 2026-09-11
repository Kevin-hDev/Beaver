use crate::models::{agent_turn_contract::NewUserTurnInput, AutomationTarget};
use crate::services::agent_local::types_message::AgentMessageKind;
use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};
use crate::services::agent_local::{conversation_admission, conversation_input, session_store};
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, NonReplayTarget, ReasoningModeId, RouteId,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

fn target() -> ContinuationTarget {
    ContinuationTarget::Forbidden(NonReplayTarget {
        route_id: RouteId::Ollama,
        model_id: "test-model".into(),
        reasoning_mode: ReasoningModeId::Off,
    })
}

#[tokio::test]
async fn automation_turn_is_appended_to_the_existing_session_and_marked() {
    let session = session_store::create_full("Existing", "test-model", "ollama", false, None)
        .await
        .unwrap();
    let input = conversation_input::resolve(NewUserTurnInput {
        content: "Vérifie la CI".into(),
        files: Vec::new(),
        skills: Vec::new(),
    })
    .await
    .unwrap();
    let profile = crate::services::reasoning_profile::EffectiveReasoningProfile::ollama(
        "test-model",
        Some("off"),
        false,
        Some(&["thinking".into()]),
    )
    .unwrap();
    let reasoning =
        crate::services::agent_local::conversation_reasoning_state::SessionReasoningUpdate::new(
            &session, &profile,
        );
    let lease =
        crate::services::agent_local::session_locks::acquire_admission_lease(&session.id).await;
    let admitted = conversation_admission::new_automation_turn_with_lease_and_reasoning(
        &lease,
        input,
        target(),
        &reasoning,
    )
    .await
    .unwrap();
    drop(lease);

    let stored = session_store::get(&session.id).await.unwrap();
    assert_eq!(stored.messages.len(), 1);
    assert_eq!(
        stored.messages[0].message_kind,
        Some(AgentMessageKind::Automation)
    );
    assert_eq!(stored.messages[0].content, "Vérifie la CI");

    let messages = crate::commands::agent_chat_task::StreamConversation::canonical_for_automation(
        admitted,
        uuid::Uuid::nil(),
        &AutomationTarget::ResumeSession {
            session_id: session.id.clone(),
        },
    )
    .into_messages()
    .unwrap();
    assert_eq!(messages[0].role, "system");
    assert!(messages[0].content.contains("resume_session"));
    assert!(messages[0].content.contains("manage_automation"));
    session_store::delete_one(&session.id).await.unwrap();
}

#[tokio::test]
async fn independent_automation_session_keeps_the_heartbeat_marker() {
    let session =
        session_store::create_with_project("Automation", "test-model", "ollama", true, None)
            .await
            .unwrap();
    assert!(session_store::get(&session.id).await.unwrap().is_heartbeat);
    session_store::delete_one(&session.id).await.unwrap();
}

#[tokio::test]
async fn occupied_session_is_not_replaced_by_background_admission() {
    let session = session_store::create_full("Busy", "test-model", "ollama", false, None)
        .await
        .unwrap();
    let existing_cancel = CancellationToken::new();
    let mut active = HashMap::new();
    active.insert(
        session.id.clone(),
        (
            existing_cancel.clone(),
            1,
            "manual-request".into(),
            Arc::new(crate::services::agent_local::parent_message_inbox::ParentMessageInbox::new()),
        ),
    );
    let app = tauri::test::mock_builder()
        .manage(crate::ActiveStreams(Mutex::new(active)))
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .unwrap();

    let result =
        crate::commands::agent_chat_admission::admit_background_if_idle(app.handle(), &session.id)
            .await;
    assert!(matches!(
        result,
        Err(crate::commands::agent_chat_admission::BackgroundAdmissionError::Busy)
    ));
    assert!(!existing_cancel.is_cancelled());
    session_store::delete_one(&session.id).await.unwrap();
}

#[tokio::test]
async fn an_error_before_the_first_durable_message_removes_only_an_empty_new_session() {
    let empty = session_store::create_full("Empty", "model", "ollama", true, None)
        .await
        .unwrap();
    super::fire::delete_empty_session(&empty.id).await;
    assert!(session_store::get(&empty.id).await.is_err());

    let retained = session_store::create_full("Retained", "test-model", "ollama", true, None)
        .await
        .unwrap();
    let input = conversation_input::resolve(NewUserTurnInput {
        content: "durable".into(),
        files: Vec::new(),
        skills: Vec::new(),
    })
    .await
    .unwrap();
    let lease =
        crate::services::agent_local::session_locks::acquire_admission_lease(&retained.id).await;
    conversation_admission::new_automation_turn_with_lease_and_reasoning(
        &lease,
        input,
        target(),
        &reasoning_update(&retained),
    )
    .await
    .unwrap();
    drop(lease);
    super::fire::delete_empty_session(&retained.id).await;
    assert!(session_store::get(&retained.id).await.is_ok());
    session_store::delete_one(&retained.id).await.unwrap();
}

#[tokio::test]
async fn tool_trace_is_persisted_before_missing_text_is_reported_in_both_modes() {
    for is_new_session in [false, true] {
        assert_tool_trace_survives_missing_text(is_new_session).await;
    }
}

async fn assert_tool_trace_survives_missing_text(is_new_session: bool) {
    let session =
        session_store::create_full("Tool trace", "test-model", "ollama", is_new_session, None)
            .await
            .unwrap();
    let input = conversation_input::resolve(NewUserTurnInput {
        content: "Inspecte le projet".into(),
        files: Vec::new(),
        skills: Vec::new(),
    })
    .await
    .unwrap();
    let lease =
        crate::services::agent_local::session_locks::acquire_admission_lease(&session.id).await;
    let admitted = conversation_admission::new_automation_turn_with_lease_and_reasoning(
        &lease,
        input,
        target(),
        &reasoning_update(&session),
    )
    .await
    .unwrap();
    drop(lease);
    let mode = if is_new_session {
        AutomationTarget::NewSession { project_id: None }
    } else {
        AutomationTarget::ResumeSession {
            session_id: session.id.clone(),
        }
    };
    let (_, journal) =
        crate::commands::agent_chat_task::StreamConversation::canonical_for_automation(
            admitted,
            uuid::Uuid::nil(),
            &mode,
        )
        .into_messages_and_journal(session.id.clone(), uuid::Uuid::new_v4().to_string())
        .unwrap();
    let mut journal = journal.unwrap();
    journal
        .persist_assistant_step(&ChatMessage::assistant(
            String::new(),
            None,
            None,
            None,
            Some(vec![ToolCallOllama {
                id: Some("call-1".into()),
                extra_content: None,
                function: ToolCallFunction {
                    name: "list_dir".into(),
                    arguments: serde_json::json!({"path": "."}),
                },
            }]),
        ))
        .await
        .unwrap();
    journal
        .persist_tool_results(
            &[ChatMessage::tool(
                "README.md".into(),
                Some("call-1".into()),
                Some("list_dir".into()),
            )],
            &[],
        )
        .await
        .unwrap();

    let saved = session_store::get(&session.id).await.unwrap();
    let roles = saved
        .messages
        .iter()
        .map(|message| message.role.as_str())
        .collect::<Vec<_>>();
    assert_eq!(roles, vec!["user", "assistant", "tool"]);
    assert!(!super::agentic::has_text_result(&[
        ChatMessage::assistant(String::new(), None, None, None, None),
        ChatMessage::tool("README.md".into(), Some("call-1".into()), None),
    ]));
    session_store::delete_one(&session.id).await.unwrap();
}

fn reasoning_update(
    session: &crate::services::agent_local::types_session::AgentSession,
) -> crate::services::agent_local::conversation_reasoning_state::SessionReasoningUpdate {
    let profile = crate::services::reasoning_profile::EffectiveReasoningProfile::ollama(
        "test-model",
        Some("off"),
        false,
        Some(&["thinking".into()]),
    )
    .unwrap();
    crate::services::agent_local::conversation_reasoning_state::SessionReasoningUpdate::new(
        session, &profile,
    )
}
