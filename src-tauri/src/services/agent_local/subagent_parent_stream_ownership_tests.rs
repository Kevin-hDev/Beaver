use super::subagent_registry;
use tokio_util::sync::CancellationToken;

fn uid() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[tokio::test]
async fn replacement_adopts_existing_children_and_current_stop_cancels_them() {
    let parent = uid();
    let adopted_child = uid();
    let new_child = uid();
    let old_owner = CancellationToken::new();
    let new_owner = CancellationToken::new();
    subagent_registry::register_execution_for_parent_stream(
        &parent,
        &adopted_child,
        CancellationToken::new(),
        None,
        &old_owner,
    )
    .await
    .expect("register old child");
    old_owner.cancel();
    subagent_registry::adopt_children_for_parent_stream(&parent, &new_owner).await;
    subagent_registry::register_execution_for_parent_stream(
        &parent,
        &new_child,
        CancellationToken::new(),
        None,
        &new_owner,
    )
    .await
    .expect("register new child");

    subagent_registry::cancel_stopped_parent_stream_children(&parent).await;
    assert!(!run(&adopted_child).await.cancelled);
    assert!(!run(&new_child).await.cancelled);

    new_owner.cancel();
    subagent_registry::cancel_stopped_parent_stream_children(&parent).await;
    assert!(run(&adopted_child).await.cancelled);
    assert!(run(&new_child).await.cancelled);
    subagent_registry::unregister(&adopted_child).await;
    subagent_registry::unregister(&new_child).await;
}

#[test]
fn chat_stream_transfers_and_cancels_parent_stream_ownership() {
    let command = include_str!("../../commands/agent_chat.rs");
    let stop_command = include_str!("../../commands/agent_chat_cancel.rs");
    let replacement = include_str!("../../commands/agent_chat_streams.rs");
    let adopt = replacement
        .find("adopt_children_for_parent_stream")
        .expect("replacement adopts children");
    let insert = replacement.find("map.insert").expect("replacement inserts winner");
    let cancel = replacement
        .find("cancel_previous(old_stream).await")
        .expect("replacement cancels loser");

    assert!(insert < adopt);
    assert!(adopt < cancel);
    assert!(command.contains("agent_chat_streams::replace_active_stream"));
    assert!(stop_command.contains("cancel_stopped_parent_stream_children"));
}

async fn run(child_id: &str) -> subagent_registry::ActiveSubagentRun {
    subagent_registry::active_run_for_child(child_id)
        .await
        .expect("active child")
}
