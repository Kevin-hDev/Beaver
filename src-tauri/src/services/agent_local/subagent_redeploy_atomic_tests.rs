use super::{session_store, subagent_registry, subagent_status};

#[test]
fn existing_delegate_uses_atomic_redeployment_helper() {
    let delegate = include_str!("tool_delegate.rs");
    let child = include_str!("tool_delegate_child.rs");
    assert!(delegate.contains("tool_delegate_child::persist_delegate_prompt"));
    assert!(delegate.contains("existing_child_id.is_some()"));
    assert!(child.contains("persist_redeployment_prompt(child_id, prompt)"));
    assert!(child.contains("unanswered_matching_prompt"));
    assert!(child.contains("session_store::add_messages"));
}

#[tokio::test]
async fn new_child_prompt_uses_message_history_not_redeployment_queue() {
    let (parent, child) = failed_child().await;

    super::tool_delegate_child::persist_delegate_prompt(&child.id, "mission initiale", false)
        .await
        .expect("persist new child prompt");
    let saved = session_store::get(&child.id).await.expect("saved new child");

    cleanup(&parent.id, &child.id).await;
    assert!(saved.subagent_queued_prompts.is_empty());
    assert!(saved
        .messages
        .iter()
        .any(|message| message.role == "user" && message.content == "mission initiale"));
}

#[tokio::test]
async fn redeploy_persists_queue_then_new_prompt_without_duplicate() {
    for (queued, prompt, expected) in [
        (
            vec!["correction durable"],
            "correction durable",
            vec!["correction durable"],
        ),
        (
            vec!["ancienne correction"],
            "nouvelle mission",
            vec!["ancienne correction", "nouvelle mission"],
        ),
    ] {
        let (parent, mut child) = failed_child().await;
        child.subagent_queued_prompts = queued.iter().map(ToString::to_string).collect();
        session_store::save(&child).await.expect("save queue");

        super::session_store_messages::add_redeployment_prompt(&child.id, prompt)
            .await
            .expect("persist redeployment atomically");
        let queued = session_store::get(&child.id).await.expect("queued redeploy");
        let run_id = subagent_registry::register(
            &parent.id,
            &child.id,
            tokio_util::sync::CancellationToken::new(),
        )
        .await
        .expect("register redeployment");
        let mut registered = queued.clone();
        registered.subagent_run_id = Some(run_id);
        session_store::save(&registered).await.expect("save run id");
        let mut context = Vec::new();
        super::subagent_instruction_delivery::drain(&child.id, &mut context)
            .await
            .expect("drain redeployment");
        let saved = session_store::get(&child.id).await.expect("saved redeploy");
        let users: Vec<&str> = saved
            .messages
            .iter()
            .filter(|message| message.role == "user")
            .map(|message| message.content.as_str())
            .collect();

        cleanup(&parent.id, &child.id).await;
        assert_eq!(queued.subagent_queued_prompts, expected);
        assert!(saved.subagent_queued_prompts.is_empty());
        assert_eq!(users, expected);
        assert_eq!(
            context
                .iter()
                .map(|message| message.content.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            saved.accumulated_tokens,
            crate::services::token_counting::estimate_agent_messages_tokens(&saved.messages)
        );
    }
}

#[tokio::test]
async fn redeploy_save_failure_keeps_queue_without_persisting_prompt() {
    let prompt = "correction durable";
    let (parent, mut child) = failed_child().await;
    child.subagent_queued_prompts.push(prompt.into());
    session_store::save(&child).await.expect("save failed child");
    let path = crate::services::paths::data_dir()
        .join("agent-sessions")
        .join(format!("{}.json", child.id));
    let backup = path.with_extension("json.redeploy-backup");
    let blocked_path = path.clone();
    let backup_path = backup.clone();

    let result = super::session_store_messages::add_redeployment_prompt_with_before_save(
        &child.id,
        prompt,
        move || async move {
            tokio::fs::rename(&blocked_path, &backup_path)
                .await
                .expect("backup child");
            tokio::fs::create_dir(&blocked_path)
                .await
                .expect("block atomic rename");
        },
    )
    .await;
    if path.is_dir() {
        tokio::fs::remove_dir(&path).await.expect("remove blocker");
    }
    tokio::fs::rename(&backup, &path)
        .await
        .expect("restore child");
    let saved = session_store::get(&child.id).await.expect("restored child");

    cleanup(&parent.id, &child.id).await;
    assert!(result.is_err());
    assert_eq!(saved.subagent_queued_prompts, vec![prompt]);
    assert!(!saved
        .messages
        .iter()
        .any(|message| message.role == "user" && message.content == prompt));
}

#[tokio::test]
async fn register_failure_keeps_redeployment_queue_durable() {
    let (parent, mut child) = failed_child().await;
    child.subagent_queued_prompts.push("ancienne correction".into());
    session_store::save(&child).await.expect("save old queue");
    super::tool_delegate_child::persist_delegate_prompt(&child.id, "nouvelle mission", true)
        .await
        .expect("persist before register");
    let active_ids = (0..4)
        .map(|_| uuid::Uuid::new_v4().to_string())
        .collect::<Vec<_>>();
    for id in &active_ids {
        subagent_registry::register_execution(
            &parent.id,
            id,
            tokio_util::sync::CancellationToken::new(),
        )
        .await
        .expect("fill parent capacity");
    }

    let result = subagent_registry::register_execution(
        &parent.id,
        &child.id,
        tokio_util::sync::CancellationToken::new(),
    )
    .await;
    let saved = session_store::get(&child.id).await.expect("saved queue");

    for id in &active_ids {
        subagent_registry::unregister(id).await;
    }
    cleanup(&parent.id, &child.id).await;
    assert!(result.is_err());
    assert_eq!(
        saved.subagent_queued_prompts,
        vec!["ancienne correction", "nouvelle mission"]
    );
}

async fn failed_child() -> (
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
    child.subagent_status = Some(subagent_status::FAILED.into());
    session_store::save(&child).await.expect("save child");
    (parent, child)
}

async fn cleanup(parent_id: &str, child_id: &str) {
    subagent_registry::unregister(child_id).await;
    session_store::delete_one(child_id).await.expect("delete child");
    session_store::delete_one(parent_id).await.expect("delete parent");
}
