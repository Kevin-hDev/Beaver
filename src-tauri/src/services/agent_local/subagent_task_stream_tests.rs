use super::*;
use crate::services::agent_local::{
    conversation_journal::ConversationJournal, session_store, subagent_registry, subagent_status,
};
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, NonReplayTarget, ReasoningModeId, RouteId,
};

#[tokio::test]
async fn cancellation_without_snapshot_keeps_durable_history() {
    let parent = session_store::create_full("Parent cancel", "llama3", "ollama", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Child cancel", "llama3", "ollama", false, None)
        .await
        .expect("create child");
    child.parent_session_id = Some(parent.id.clone());
    child.subagent_type = Some("explorer".into());
    child.subagent_status = Some(subagent_status::RUNNING.into());
    child.messages.push(super::super::subagent_instruction_delivery::agent_message(
        "mission durable",
    ));
    let registered = subagent_registry::register_execution(
        &parent.id,
        &child.id,
        CancellationToken::new(),
    )
    .await
    .expect("register child");
    child.subagent_run_id = Some(registered.run_id.clone());
    session_store::save(&child).await.expect("save child");
    let cancel = CancellationToken::new();
    cancel.cancel();
    let (success, status, summary) =
        finalize_stream_result(Err("Annulé".into()), &child.id, "request", cancel)
            .await
            .expect("cancel outcome");

    let saved = session_store::get(&child.id).await.expect("load child");
    assert!(!success);
    assert_eq!(status, subagent_status::CANCELLED);
    assert_eq!(summary, "Sous-agent annulé.");
    assert_eq!(saved.messages.len(), 1);
    assert_eq!(saved.messages[0].content, "mission durable");
    subagent_registry::unregister(&child.id).await;
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id).await.expect("delete parent");
}

#[tokio::test]
async fn subagent_conversation_adoption_reloads_the_canonical_history() {
    let parent = session_store::create_full("Parent adoption", "model", "groq", false, None)
        .await
        .expect("create parent");
    let mut child = session_store::create_full("Child adoption", "model", "groq", false, None)
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
    let target = ContinuationTarget::Forbidden(NonReplayTarget {
        route_id: RouteId::Groq,
        model_id: "model".into(),
        reasoning_mode: ReasoningModeId::Off,
    });

    let admitted = admit_subagent_turn(
        &child.id,
        "mission durable",
        target,
        &registered.run_id,
        &registered.execution_id,
    )
    .await
    .expect("admit child prompt");
    let mut journal = ConversationJournal::new_for_subagent(
        child.id.clone(),
        admitted.turn_id.clone(),
        admitted.user_message_id.clone(),
        admitted.assistant_message_id.clone(),
        uuid::Uuid::new_v4().to_string(),
        registered.run_id.clone(),
        registered.execution_id.clone(),
    )
    .expect("create child journal");
    journal
        .persist_assistant_step(&super::super::types_ollama::ChatMessage::assistant(
            "réponse durable".into(),
            None,
            None,
            None,
            None,
        ))
        .await
        .expect("persist child assistant");
    journal.commit_turn().await.expect("commit child turn");

    let reloaded = session_store::get(&child.id).await.expect("reload child");
    assert_eq!(reloaded.messages.len(), 2);
    assert_eq!(reloaded.messages[0].id, admitted.user_message_id);
    assert_eq!(reloaded.messages[0].turn_id, admitted.turn_id);
    assert_eq!(reloaded.messages[1].id, admitted.assistant_message_id);
    assert_eq!(reloaded.messages[1].turn_id, admitted.turn_id);
    assert!(reloaded
        .messages
        .iter()
        .all(|message| message.continuation.is_none()));

    subagent_registry::unregister(&child.id).await;
    session_store::delete_one(&child.id).await.expect("delete child");
    session_store::delete_one(&parent.id).await.expect("delete parent");
}
