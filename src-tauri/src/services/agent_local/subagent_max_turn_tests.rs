use super::subagent_orchestration::ParentSubagentOrchestrator;
use super::types_ollama::ChatMessage;
use super::{session_store, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn last_turn_keeps_correction_unconsumed_and_context_untouched() {
    let root = session_store::create_full("Root", "llama3", "ollama", false, None)
        .await
        .expect("create root");
    let mut child = session_store::create_full("Child", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(root.id.clone());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    child.subagent_queued_prompts.push("correction finale".into());
    let registered = subagent_registry::register_execution(
        &root.id,
        &child.id,
        CancellationToken::new(),
    )
    .await
    .expect("register child");
    child.subagent_run_id = Some(registered.run_id.clone());
    session_store::save(&child).await.expect("save child");
    let mut orchestrator = ParentSubagentOrchestrator::new(&child.id).await;
    let messages = [ChatMessage {
        role: "assistant".into(),
        content: "réponse terminale".into(),
        ..Default::default()
    }];

    let result = orchestrator.ensure_no_followup_at_turn_limit().await;

    let saved = session_store::get(&child.id).await.expect("load child");
    assert_eq!(result, Err(super::agent_loop_errors::max_turns_message()));
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].content, "réponse terminale");
    assert_eq!(saved.subagent_queued_prompts, vec!["correction finale"]);
    assert!(!subagent_registry::prompt_was_delivered(
        &child.id,
        &registered.execution_id,
        "correction finale",
    )
    .await);
    cleanup(&root.id, &child.id).await;
}

#[tokio::test]
async fn last_turn_without_followup_finishes_normally() {
    let session = session_store::create_full("Final", "llama3", "ollama", false, None)
        .await
        .expect("create session");
    let mut orchestrator = ParentSubagentOrchestrator::new(&session.id).await;

    let result = orchestrator.ensure_no_followup_at_turn_limit().await;

    assert_eq!(result, Ok(()));
    session_store::delete_one(&session.id)
        .await
        .expect("delete session");
}

#[test]
fn api_and_ollama_share_the_last_turn_guard() {
    for source in [
        include_str!("agent_loop.rs"),
        include_str!("../llm/agent_loop.rs"),
    ] {
        assert!(source.contains("turn + 1 < MAX_TURNS"));
        assert!(source.contains("continue_after_no_tool_turn"));
    }
}

async fn cleanup(parent_id: &str, child_id: &str) {
    subagent_registry::unregister(child_id).await;
    session_store::delete_one(child_id).await.expect("delete child");
    session_store::delete_one(parent_id).await.expect("delete parent");
}
