use super::{session_store, subagent_instruction_delivery, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn orphan_child_without_registry_fails_closed() {
    let (parent, mut child) = child_session(subagent_status::RUNNING).await;
    child.subagent_queued_prompts.push("orphan correction".into());
    session_store::save(&child).await.expect("save orphan");
    let mut context = Vec::new();

    let result = subagent_instruction_delivery::drain(&child.id, &mut context).await;
    let saved = session_store::get(&child.id).await.expect("saved orphan");

    cleanup(&parent.id, &child.id).await;
    assert!(result.is_err());
    assert!(context.is_empty());
    assert_eq!(saved.subagent_queued_prompts, vec!["orphan correction"]);
}

#[tokio::test]
async fn persisted_queue_over_capacity_is_rejected_before_injection() {
    let (parent, mut child) = child_session(subagent_status::RUNNING).await;
    child.subagent_queued_prompts = (0..9).map(|index| format!("correction {index}")).collect();
    let run_id = subagent_registry::register(&parent.id, &child.id, CancellationToken::new())
        .await
        .expect("register child");
    child.subagent_run_id = Some(run_id);
    session_store::save(&child).await.expect("save corrupt queue");
    let mut context = Vec::new();

    let result = subagent_instruction_delivery::drain(&child.id, &mut context).await;
    let saved = session_store::get(&child.id).await.expect("saved child");

    cleanup(&parent.id, &child.id).await;
    assert!(result.is_err());
    assert!(context.is_empty());
    assert_eq!(saved.subagent_queued_prompts.len(), 9);
}

#[tokio::test]
async fn normalized_duplicate_queue_is_rejected_without_injection() {
    let (parent, mut child) = child_session(subagent_status::RUNNING).await;
    child.subagent_queued_prompts = vec!["corrige".into(), "  corrige  ".into()];
    activate(&parent.id, &mut child).await;
    let mut context = Vec::new();

    let result = subagent_instruction_delivery::drain(&child.id, &mut context).await;
    let saved = session_store::get(&child.id).await.expect("saved child");

    cleanup(&parent.id, &child.id).await;
    assert!(result.is_err());
    assert!(context.is_empty());
    assert_eq!(saved.subagent_queued_prompts, vec!["corrige", "  corrige  "]);
}

#[tokio::test]
async fn queue_accepts_exact_item_and_character_limits() {
    let (parent, mut child) = child_session(subagent_status::RUNNING).await;
    child.subagent_queued_prompts = (0..8)
        .map(|index| {
            if index == 7 {
                "x".repeat(50_000)
            } else {
                format!("correction {index}")
            }
        })
        .collect();
    activate(&parent.id, &mut child).await;
    let mut context = Vec::new();

    let result = subagent_instruction_delivery::drain(&child.id, &mut context).await;

    cleanup(&parent.id, &child.id).await;
    assert_eq!(result.expect("exact limits are valid"), 8);
    assert_eq!(context.len(), 8);
    assert_eq!(context[7].content.chars().count(), 50_000);
}

#[tokio::test]
async fn queue_rejects_prompt_over_character_limit_without_mutation() {
    let (parent, mut child) = child_session(subagent_status::RUNNING).await;
    child.subagent_queued_prompts = vec!["x".repeat(50_001)];
    activate(&parent.id, &mut child).await;
    let mut context = Vec::new();

    let result = subagent_instruction_delivery::drain(&child.id, &mut context).await;
    let saved = session_store::get(&child.id).await.expect("saved child");

    cleanup(&parent.id, &child.id).await;
    assert!(result.is_err());
    assert!(context.is_empty());
    assert_eq!(saved.subagent_queued_prompts[0].chars().count(), 50_001);
}

#[tokio::test]
async fn completed_child_with_invalid_type_never_defaults_to_explorer() {
    for invalid_type in [None, Some("corrupted")] {
        let (parent, mut child) = child_session(subagent_status::COMPLETED).await;
        child.subagent_type = invalid_type.map(str::to_string);
        session_store::save(&child).await.expect("save invalid type");

        let result = super::tool_subagent_message::run(
            &serde_json::json!({"subagent_id": child.id, "prompt": "reprends"}),
            &parent.id,
        )
        .await;

        cleanup(&parent.id, &child.id).await;
        assert!(result.is_error);
        assert!(result.content.contains("delegate_task"));
        assert!(result.content.contains("subagent_id"));
    }
}

async fn child_session(status: &str) -> (
    super::types_session::AgentSession,
    super::types_session::AgentSession,
) {
    let parent = session_store::create_full("Parent", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Geminitor", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(status.into());
    session_store::save(&child).await.expect("save child");
    (parent, child)
}

async fn cleanup(parent_id: &str, child_id: &str) {
    subagent_registry::unregister(child_id).await;
    session_store::delete_one(child_id).await.expect("delete child");
    session_store::delete_one(parent_id).await.expect("delete parent");
}

async fn activate(parent_id: &str, child: &mut super::types_session::AgentSession) {
    let run_id = subagent_registry::register(parent_id, &child.id, CancellationToken::new())
        .await
        .expect("register child");
    child.subagent_run_id = Some(run_id);
    session_store::save(child).await.expect("save active child");
}
