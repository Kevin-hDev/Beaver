use super::{session_store, subagent_registry, subagent_status, tool_delegate_child};

#[test]
fn delegate_prompt_validation_preserves_original_bytes() {
    let original = "  garde les espaces\n    et ce bloc indenté\n";
    let args = serde_json::json!({"prompt": original});

    let parsed = super::tool_delegate_prompt::from_args(&args).expect("valid prompt");

    assert_eq!(parsed.as_bytes(), original.as_bytes());
}

#[tokio::test]
async fn prompt_preflight_never_writes_a_user_turn() {
    let (parent, child) = failed_child().await;
    let prompt = "mission identique";
    tool_delegate_child::persist_delegate_prompt(&child.id, prompt)
        .await
        .expect("preflight initial prompt");
    let saved = session_store::get(&child.id).await.expect("load child");
    assert!(saved.messages.is_empty());
    assert!(saved.subagent_queued_prompts.is_empty());

    cleanup(&parent.id, &child.id).await;
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
    session_store::delete_one(child_id)
        .await
        .expect("delete child");
    session_store::delete_one(parent_id)
        .await
        .expect("delete parent");
}
