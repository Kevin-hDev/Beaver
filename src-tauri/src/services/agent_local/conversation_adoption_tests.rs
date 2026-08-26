use super::conversation_history::ProviderRole;
use super::session_store;
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, NonReplayTarget, ReasoningModeId, RouteId,
};

#[tokio::test]
async fn scheduler_conversation_adoption_persists_the_canonical_turn_once() {
    let session = session_store::create_full("Wakeup", "model", "groq", true, None)
        .await
        .expect("create session");
    let target = ContinuationTarget::Forbidden(NonReplayTarget {
        route_id: RouteId::Groq,
        model_id: "model".into(),
        reasoning_mode: ReasoningModeId::Off,
    });

    let admitted = crate::services::scheduler::admit_wakeup_turn(
        &session.id,
        "Inspecte le projet",
        target,
    )
    .await
    .expect("admit wakeup");
    let saved = session_store::get(&session.id).await.expect("reload session");

    assert_eq!(saved.messages.len(), 1);
    assert_eq!(saved.messages[0].id, admitted.user_message_id);
    assert_eq!(saved.messages[0].turn_id, admitted.turn_id);
    assert_eq!(saved.messages[0].role, "user");
    assert_eq!(admitted.history.messages[0].role, ProviderRole::User);
    assert_eq!(admitted.history.messages[0].content, "Inspecte le projet");

    session_store::delete_one(&session.id).await.expect("delete session");
}

#[test]
fn every_internal_entry_point_adopts_a_canonical_conversation() {
    let scheduler = include_str!("../scheduler/agentic.rs");
    let gateway = include_str!("../gateway/agent_bridge.rs");
    let subagent = include_str!("subagent_task_stream.rs");
    let chat = include_str!("../../commands/agent_chat_run_spawn.rs");

    for source in [scheduler, gateway, subagent, chat] {
        assert!(source.contains("StreamConversation::canonical"));
        assert!(!source.contains("internal_legacy"));
        assert!(!source.contains("session_store::add_messages"));
    }
    assert!(!include_str!("../../commands/agent_chat_task/conversation.rs")
        .contains("InternalLegacy"));
}
