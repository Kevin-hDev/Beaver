use super::subagent_read_only_command_test_support::{
    assert_rejected, child_session, cleanup, snapshot, user_message, SUBAGENT_READ_ONLY,
};
use crate::services::agent_local::session_permission_state;
use crate::services::agent_local::session_store;
use crate::services::agent_local::tool_plan;

#[tokio::test]
async fn assign_session_project_rejects_a_child_without_persisting_the_project() {
    let session = child_session("Save").await;
    let before = snapshot(&session.id).await;

    assert_rejected(
        &session,
        &before,
        super::agent_sessions::assign_session_project(
            session.id.clone(),
            "blocked-project".to_string(),
        ),
    )
    .await;
}

#[tokio::test]
async fn rename_agent_session_rejects_a_child_without_persisting_the_name() {
    let session = child_session("Original child name").await;
    let before = snapshot(&session.id).await;

    assert_rejected(
        &session,
        &before,
        super::agent_sessions::rename_agent_session(session.id.clone(), "User rename".to_string()),
    )
    .await;
}

#[tokio::test]
async fn add_messages_to_session_rejects_a_child_without_persisting_history() {
    let session = child_session("History").await;
    let before = snapshot(&session.id).await;
    let message = user_message("Blocked history mutation");

    assert_rejected(
        &session,
        &before,
        super::agent_sessions::add_messages_to_session(
            session.id.clone(),
            vec![message],
            1,
            None,
            None,
        ),
    )
    .await;
}

#[tokio::test]
async fn truncate_and_replace_at_rejects_a_child_without_changing_history() {
    let mut session = child_session("Retry").await;
    let existing = user_message("Original message");
    let message_id = existing.id.clone();
    session.messages.push(existing);
    session_store::save(&session).await.expect("seed message");
    let before = snapshot(&session.id).await;

    assert_rejected(
        &session,
        &before,
        super::agent_sessions::truncate_and_replace_at(
            session.id.clone(),
            message_id,
            Some(user_message("Replacement message")),
        ),
    )
    .await;
}

#[tokio::test]
async fn set_session_permission_mode_rejects_a_child_without_persisting_mode() {
    let session = child_session("Permission").await;
    let before = snapshot(&session.id).await;
    let before_state = session_permission_state::load(&session.id)
        .await
        .expect("permission state");
    let error = super::agent_sessions::set_session_permission_mode(
        session.id.clone(),
        "manual".to_string(),
    )
    .await
    .err();
    let after = snapshot(&session.id).await;
    let after_state = session_permission_state::load(&session.id)
        .await
        .expect("permission state after refusal");
    cleanup(&session).await;

    assert_eq!(error.as_deref(), Some(SUBAGENT_READ_ONLY));
    assert_eq!(after, before);
    assert_eq!(after_state, before_state);
}

#[tokio::test]
async fn update_session_model_rejects_a_child_without_persisting_model() {
    let session = child_session("Model").await;
    let before = snapshot(&session.id).await;

    assert_rejected(
        &session,
        &before,
        super::agent_sessions::update_session_model(
            session.id.clone(),
            "updated-model".to_string(),
            "updated-provider".to_string(),
            Some("high".to_string()),
            Some(true),
        ),
    )
    .await;
}

#[tokio::test]
async fn update_session_reasoning_rejects_a_child_without_persisting_reasoning() {
    let session = child_session("Reasoning").await;
    let before = snapshot(&session.id).await;

    assert_rejected(
        &session,
        &before,
        super::agent_sessions::update_session_reasoning(
            session.id.clone(),
            Some("high".to_string()),
            Some(true),
        ),
    )
    .await;
}

#[tokio::test]
async fn set_session_plan_mode_rejects_a_child_without_persisting_plan_state() {
    let session = child_session("Plan").await;
    let before = snapshot(&session.id).await;
    let before_enabled = tool_plan::is_enabled(&session.id).await;
    let error = super::agent_sessions::set_session_plan_mode(session.id.clone(), true)
        .await
        .err();
    let after = snapshot(&session.id).await;
    let after_enabled = tool_plan::is_enabled(&session.id).await;
    cleanup(&session).await;

    assert_eq!(error.as_deref(), Some(SUBAGENT_READ_ONLY));
    assert_eq!(after, before);
    assert_eq!(after_enabled, before_enabled);
}
