use crate::models::agent_session_contract::VisibleMessageInput;
use crate::services::agent_local::session_permission_state;
use crate::services::agent_local::session_store;
pub(super) use crate::services::agent_local::session_user_write::SUBAGENT_READ_ONLY;
use crate::services::agent_local::types_session::{AgentMessage, AgentSession};
use std::future::Future;

pub(super) async fn child_session(name: &str) -> AgentSession {
    let mut session = session_store::create_full(name, "model", "provider", false, None)
        .await
        .expect("session");
    session.parent_session_id = Some(uuid::Uuid::new_v4().to_string());
    session_store::save(&session).await.expect("save child");
    session
}

pub(super) async fn snapshot(session_id: &str) -> serde_json::Value {
    serde_json::to_value(
        session_store::get(session_id)
            .await
            .expect("reload session"),
    )
    .expect("serialize session")
}

pub(super) async fn assert_rejected<T>(
    session: &AgentSession,
    before: &serde_json::Value,
    action: impl Future<Output = Result<T, String>>,
) {
    let error = action.await.err();
    let after = snapshot(&session.id).await;
    cleanup(session).await;

    assert_eq!(error.as_deref(), Some(SUBAGENT_READ_ONLY));
    assert_eq!(&after, before);
}

pub(super) async fn cleanup(session: &AgentSession) {
    session_permission_state::remove(&session.id).await;
    session_store::delete_one(&session.id)
        .await
        .expect("cleanup session");
}

pub(super) fn user_message(content: &str) -> AgentMessage {
    AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        turn_id: AgentMessage::new_turn_id(),
        role: "user".to_string(),
        content: content.to_string(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        continuation: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: chrono::Utc::now(),
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    }
}

pub(super) fn visible_user_message(content: &str) -> VisibleMessageInput {
    let message = user_message(content);
    VisibleMessageInput {
        id: message.id,
        role: message.role,
        content: message.content,
        thinking: None,
        tool_calls: None,
        tool_name: None,
        tool_call_id: None,
        tool_activities: None,
        segments: None,
        files: Vec::new(),
        timestamp: message.timestamp,
        tokens: 0,
        work_duration_ms: None,
        skill_names: None,
        stream_run_id: None,
        stream_part: None,
    }
}
