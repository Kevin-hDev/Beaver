use super::{
    session_store, subagent_completion, subagent_hidden_reports, subagent_registry,
    subagent_status, tool_delegate_child,
};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn failed_run_keeps_correction_until_explicit_redeployment() {
    let parent = session_store::create_full("Parent", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Geminitor", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    child.subagent_queued_prompts.push("correction durable".into());
    session_store::save(&child).await.expect("save child");
    subagent_registry::register(&parent.id, &child.id, CancellationToken::new())
        .await
        .expect("register failed run");

    subagent_completion::persist_terminal_completion(
        &parent.id,
        &child.id,
        "explorer",
        subagent_status::FAILED,
        "Échec provider",
    )
    .await
    .expect("persist failed completion");

    let failed = session_store::get(&child.id).await.expect("load failed child");
    assert_eq!(failed.subagent_queued_prompts, vec!["correction durable"]);
    assert_eq!(failed.subagent_status.as_deref(), Some(subagent_status::FAILED));
    assert_eq!(subagent_hidden_reports::peek_reports(&parent.id).await.len(), 1);
    assert!(subagent_registry::active_children_for_parent(&parent.id)
        .await
        .is_empty());

    let run_id = subagent_registry::get_or_create_run_id(&parent.id).await;
    tool_delegate_child::prepare_existing_child(
        &child.id,
        &parent.id,
        "explorer",
        "correction durable",
        "Geminitor",
        "Reprise",
        "geminitor",
        &run_id,
    )
    .await
    .expect("prepare explicit redeployment");
    let persisted = tool_delegate_child::persist_delegate_prompt(
        &child.id,
        "correction durable",
        true,
    )
    .await
    .expect("reuse queued correction");
    let registered = subagent_registry::register_execution_with_initial_prompt(
        &parent.id,
        &child.id,
        CancellationToken::new(),
        persisted.initial_prompt(),
    )
    .await
    .expect("register redeployment");
    let mut context = Vec::new();
    super::subagent_instruction_delivery::drain(&child.id, &mut context)
        .await
        .expect("deliver correction once");

    let resumed = session_store::get(&child.id).await.expect("load resumed child");
    assert!(resumed.subagent_queued_prompts.is_empty());
    assert_eq!(
        resumed
            .messages
            .iter()
            .filter(|message| message.role == "user" && message.content == "correction durable")
            .count(),
        1
    );
    assert_eq!(context.len(), 1);
    assert!(subagent_registry::prompt_was_delivered(
        &child.id,
        &registered.execution_id,
        "correction durable",
    )
    .await);

    subagent_registry::unregister(&child.id).await;
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id).await.expect("delete parent");
}
