use super::{session_store, subagent_archive, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn archived_child_cannot_be_redeployed() {
    let (parent, child) = sessions("archived", subagent_status::COMPLETED).await;

    let outcome = subagent_archive::archive_owned(&child.id, &parent.id)
        .await
        .expect("archive child");
    let run_id = subagent_registry::get_or_create_run_id(&parent.id).await;
    let redeploy = super::tool_delegate_child::prepare_existing_child(
        &child.id,
        &parent.id,
        "explorer",
        "Nouvelle mission",
        "Geminitor",
        "Reprise",
        "geminitor",
        &run_id,
    )
    .await;

    assert_eq!(outcome, subagent_archive::ArchiveOutcome::Archived);
    assert!(redeploy.is_err());
    assert!(session_store::get(&child.id)
        .await
        .expect("load child")
        .archived_at
        .is_some());
    subagent_registry::release_run_claim(&parent.id, &run_id).await;
    cleanup(&parent.id, &child.id).await;
}

#[tokio::test]
async fn active_child_cannot_be_archived() {
    let (parent, mut child) = sessions("active", subagent_status::RUNNING).await;
    let registered = subagent_registry::register_execution(
        &parent.id,
        &child.id,
        CancellationToken::new(),
    )
    .await
    .expect("register child");
    child.subagent_run_id = Some(registered.run_id.clone());
    session_store::save(&child).await.expect("save active child");

    let outcome = subagent_archive::archive_owned(&child.id, &parent.id)
        .await
        .expect("check active child");

    assert_eq!(outcome, subagent_archive::ArchiveOutcome::Active);
    assert!(session_store::get(&child.id)
        .await
        .expect("load child")
        .archived_at
        .is_none());
    assert!(subagent_registry::owns_execution(
        &child.id,
        &registered.run_id,
        &registered.execution_id,
    )
    .await);
    cleanup(&parent.id, &child.id).await;
}

async fn sessions(
    suffix: &str,
    status: &str,
) -> (
    super::types_session::AgentSession,
    super::types_session::AgentSession,
) {
    let parent = session_store::create_full(
        &format!("Parent {suffix}"),
        "llama3",
        "ollama",
        false,
        None,
    )
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
