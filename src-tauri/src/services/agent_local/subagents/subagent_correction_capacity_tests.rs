use super::{session_store, subagent_instruction_delivery, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn sixty_four_corrections_are_allowed_and_sixty_fifth_is_rejected() {
    let parent = session_store::create_full("Parent capacity", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Child capacity", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    let registered = subagent_registry::register_execution(
        &parent.id,
        &child.id,
        CancellationToken::new(),
    )
    .await
    .expect("register child");
    child.subagent_run_id = Some(registered.run_id.clone());
    session_store::save(&child).await.expect("save child");

    for batch in 0..8 {
        let mut saved = session_store::get(&child.id).await.expect("load child");
        saved.subagent_queued_prompts = (0..8)
            .map(|index| format!("correction {}", batch * 8 + index))
            .collect();
        session_store::save(&saved).await.expect("save batch");
        assert_eq!(
            subagent_instruction_delivery::drain(&child.id, &mut Vec::new())
                .await
                .expect("drain allowed batch"),
            8
        );
    }
    assert!(subagent_registry::prompt_was_delivered(
        &child.id,
        &registered.execution_id,
        "correction 63",
    )
    .await);

    let mut saved = session_store::get(&child.id).await.expect("load at cap");
    saved.subagent_queued_prompts = vec!["correction 64".into()];
    session_store::save(&saved).await.expect("save over-cap queue");
    let result = subagent_instruction_delivery::drain(&child.id, &mut Vec::new()).await;
    let still_queued = session_store::get(&child.id).await.expect("load rejected queue");
    assert!(result.is_err());
    assert_eq!(still_queued.subagent_queued_prompts, vec!["correction 64"]);

    let mut cleared = still_queued;
    cleared.subagent_queued_prompts.clear();
    session_store::save(&cleared).await.expect("clear test queue");
    let tool_result = super::tool_subagent_message::run(
        &serde_json::json!({"subagent_id": child.id, "prompt": "correction 64"}),
        &parent.id,
    )
    .await;
    let final_child = session_store::get(&child.id).await.expect("load final child");
    assert!(tool_result.is_error);
    assert!(final_child.subagent_queued_prompts.is_empty());

    subagent_registry::unregister(&child.id).await;
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id).await.expect("delete parent");
}
