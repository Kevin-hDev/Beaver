use super::{session_store, subagent_registry, subagent_status, tool_delegate_child};
use tokio_util::sync::CancellationToken;

#[test]
fn delegate_prompt_validation_preserves_original_bytes() {
    let original = "  garde les espaces\n    et ce bloc indenté\n";
    let args = serde_json::json!({"prompt": original});

    let parsed = super::tool_delegate_prompt::from_args(&args).expect("valid prompt");

    assert_eq!(parsed.as_bytes(), original.as_bytes());
}

#[tokio::test]
async fn initial_prompt_is_registered_as_delivered_with_normalized_duplicates() {
    let parent_id = uuid::Uuid::new_v4().to_string();
    let child_id = uuid::Uuid::new_v4().to_string();
    let prompt = "mission   initiale".to_string();
    subagent_registry::get_or_create_run_id(&parent_id).await;

    let registered = subagent_registry::register_execution_with_initial_prompt(
        &parent_id,
        &child_id,
        CancellationToken::new(),
        Some(&prompt),
    )
    .await
    .expect("register delivered initial prompt");

    assert!(subagent_registry::prompt_was_delivered(
        &child_id,
        &registered.execution_id,
        "  mission initiale  ",
    )
    .await);
    subagent_registry::unregister(&child_id).await;
}

#[tokio::test]
async fn unanswered_retry_reuses_user_turn_but_answered_retry_is_queued() {
    let (parent, child) = failed_child().await;
    let prompt = "mission identique";
    tool_delegate_child::persist_delegate_prompt(&child.id, prompt, false)
        .await
        .expect("persist initial prompt");

    let retry = tool_delegate_child::persist_delegate_prompt(
        &child.id,
        "  mission   identique ",
        true,
    )
    .await
    .expect("reuse unanswered prompt");
    assert!(matches!(
        retry,
        tool_delegate_child::DelegatePromptPersistence::AlreadyDelivered(_)
    ));
    let unanswered = session_store::get(&child.id).await.expect("load unanswered");
    assert!(unanswered.subagent_queued_prompts.is_empty());
    assert_eq!(user_turn_count(&unanswered, prompt), 1);

    session_store::add_messages(&child.id, vec![message("assistant", "réponse")], 0)
        .await
        .expect("persist answer");
    let retry = tool_delegate_child::persist_delegate_prompt(&child.id, prompt, true)
        .await
        .expect("queue answered retry");
    assert!(matches!(
        retry,
        tool_delegate_child::DelegatePromptPersistence::Queued
    ));
    let answered = session_store::get(&child.id).await.expect("load answered");
    assert_eq!(answered.subagent_queued_prompts, vec![prompt]);

    cleanup(&parent.id, &child.id).await;
}

fn user_turn_count(session: &super::types_session::AgentSession, prompt: &str) -> usize {
    session
        .messages
        .iter()
        .filter(|message| message.role == "user" && message.content == prompt)
        .count()
}

fn message(role: &str, content: &str) -> super::types_session::AgentMessage {
    super::types_session::AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: role.to_string(),
        content: content.to_string(),
        thinking: None,
        tool_calls: None,
        tool_name: None,
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

async fn failed_child() -> (
    super::types_session::AgentSession,
    super::types_session::AgentSession,
) {
    let parent = session_store::create_full("Parent prompt", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Child prompt", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(subagent_status::FAILED.into());
    session_store::save(&child).await.expect("save child");
    (parent, child)
}

async fn cleanup(parent_id: &str, child_id: &str) {
    subagent_registry::unregister(child_id).await;
    session_store::delete_one(child_id).await.expect("delete child");
    session_store::delete_one(parent_id).await.expect("delete parent");
}
