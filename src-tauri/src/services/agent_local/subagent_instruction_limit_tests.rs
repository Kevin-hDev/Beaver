use super::{session_store, subagent_registry, subagent_status};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn drain_keeps_session_messages_bounded_at_two_thousand() {
    let parent = session_store::create_full("Parent limit", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Geminitor", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    child.messages = (0..1_999)
        .map(|index| agent_message(&format!("ancien {index}")))
        .collect();
    child.subagent_queued_prompts = vec!["correction 1".into(), "correction 2".into()];
    session_store::save(&child).await.expect("save child");
    let run_id = subagent_registry::register(&parent.id, &child.id, CancellationToken::new())
        .await
        .expect("register child");
    child.subagent_run_id = Some(run_id);
    session_store::save(&child).await.expect("save run id");

    super::subagent_instruction_delivery::drain(&child.id, &mut Vec::new())
        .await
        .expect("drain instructions");
    let saved = session_store::get(&child.id).await.expect("saved child");

    subagent_registry::unregister(&child.id).await;
    session_store::delete_one(&child.id)
        .await
        .expect("delete child");
    session_store::delete_one(&parent.id)
        .await
        .expect("delete parent");
    assert_eq!(saved.messages.len(), 2_000);
    assert_eq!(saved.messages[1_998].content, "correction 1");
    assert_eq!(saved.messages[1_999].content, "correction 2");
}

fn agent_message(content: &str) -> super::types_session::AgentMessage {
    super::types_session::AgentMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "assistant".into(),
        content: content.into(),
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
