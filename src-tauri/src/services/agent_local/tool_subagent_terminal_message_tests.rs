use super::*;
use crate::services::agent_local::{session_store, subagent_registry, subagent_status};
use serde_json::json;

#[tokio::test]
async fn terminal_failure_statuses_require_delegate_task_with_existing_id() {
    for status in [
        subagent_status::FAILED,
        subagent_status::CANCELLED,
        subagent_status::INTERRUPTED,
    ] {
        assert_redeploy_required(Some(status)).await;
    }
}

#[tokio::test]
async fn missing_or_invalid_status_fails_closed_instead_of_resuming() {
    assert_redeploy_required(None).await;
    assert_redeploy_required(Some("corrupted-status")).await;
}

#[tokio::test]
async fn orphan_running_status_fails_closed_without_adding_to_queue() {
    let (parent, child) = inactive_child(Some(subagent_status::RUNNING)).await;

    let result = message(
        &json!({ "subagent_id": child.id, "prompt": "reprends" }),
        &parent.id,
    )
    .await;
    let saved = session_store::get(&child.id).await.expect("saved child");

    cleanup(&parent.id, &child.id).await;
    assert!(result.is_error);
    assert!(result.content.contains("delegate_task"));
    assert!(saved.subagent_queued_prompts.is_empty());
}

#[tokio::test]
async fn completed_child_builds_delegate_payload_with_existing_subagent_id() {
    let (parent, child) = inactive_child(Some(subagent_status::COMPLETED)).await;

    let payload = build_resume_payload(&child, "reprends explicitement")
        .expect("valid explicit resume payload");

    cleanup(&parent.id, &child.id).await;
    assert_eq!(payload["subagent_id"], child.id);
    assert_eq!(payload["subagent_type"], "explorer");
    assert_eq!(payload["prompt"], "reprends explicitement");
}

#[tokio::test]
async fn active_registry_run_mismatch_refuses_even_completed_session() {
    let (parent, mut child) = inactive_child(Some(subagent_status::COMPLETED)).await;
    let _active_run = subagent_registry::register(
        &parent.id,
        &child.id,
        tokio_util::sync::CancellationToken::new(),
    )
    .await
    .expect("register active run");
    child.subagent_run_id = Some("stale-run".into());
    session_store::save(&child).await.expect("save stale run");

    let result = message(
        &json!({ "subagent_id": child.id, "prompt": "reprends" }),
        &parent.id,
    )
    .await;

    cleanup(&parent.id, &child.id).await;
    assert!(result.is_error);
    assert!(result.content.contains("delegate_task"));
}

async fn assert_redeploy_required(status: Option<&str>) {
    let (parent, child) = inactive_child(status).await;
    let result = message(
        &json!({ "subagent_id": child.id, "prompt": "reprends" }),
        &parent.id,
    )
    .await;

    cleanup(&parent.id, &child.id).await;
    assert!(result.is_error);
    assert!(result.content.contains("delegate_task"));
    assert!(result.content.contains("subagent_id"));
}

async fn inactive_child(status: Option<&str>) -> (
    crate::services::agent_local::types_session::AgentSession,
    crate::services::agent_local::types_session::AgentSession,
) {
    let parent = session_store::create_full("Parent", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Geminitor", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = status.map(str::to_string);
    session_store::save(&child).await.expect("save child");
    (parent, child)
}

async fn cleanup(parent_id: &str, child_id: &str) {
    subagent_registry::unregister(child_id).await;
    session_store::delete_one(child_id).await.expect("delete child");
    session_store::delete_one(parent_id)
        .await
        .expect("delete parent");
}
