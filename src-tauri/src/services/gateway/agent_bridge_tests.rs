use super::admit_gateway_turn;
use crate::services::agent_local::types_ollama::ChatMessage;
use crate::services::agent_local::{conversation_journal::ConversationJournal, session_store};
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, NonReplayTarget, ReasoningModeId, RouteId,
};

#[tokio::test]
async fn gateway_conversation_adoption_persists_the_canonical_turn_once() {
    let session = session_store::create_gateway(
        "Gateway adoption",
        "gateway-model",
        "groq",
        format!("gateway-adoption-{}", uuid::Uuid::new_v4()),
    )
    .await
    .expect("create gateway session");
    let target = ContinuationTarget::Forbidden(NonReplayTarget {
        route_id: RouteId::Groq,
        model_id: "gateway-model".into(),
        reasoning_mode: ReasoningModeId::Off,
    });

    let admitted = admit_gateway_turn(&session.id, "durable inbound", target)
        .await
        .expect("admit inbound gateway message");
    let mut journal = ConversationJournal::new(
        session.id.clone(),
        admitted.turn_id.clone(),
        admitted.user_message_id.clone(),
        admitted.assistant_message_id.clone(),
        uuid::Uuid::new_v4().to_string(),
    )
    .expect("create canonical journal");
    journal
        .persist_assistant_step(&ChatMessage::assistant(
            "durable outbound".into(),
            None,
            None,
            None,
            None,
        ))
        .await
        .expect("persist assistant before reply");
    journal.commit_turn().await.expect("commit canonical turn");

    let stored = session_store::get(&session.id)
        .await
        .expect("reload session");
    assert_eq!(stored.schema_version, 2);
    assert_eq!(stored.messages.len(), 2);
    assert_eq!(stored.messages[0].id, admitted.user_message_id);
    assert_eq!(stored.messages[0].turn_id, admitted.turn_id);
    assert_eq!(stored.messages[0].content, "durable inbound");
    assert!(stored.messages[0].continuation.is_none());
    assert_eq!(stored.messages[1].id, admitted.assistant_message_id);
    assert_eq!(stored.messages[1].turn_id, admitted.turn_id);
    assert_eq!(stored.messages[1].content, "durable outbound");
    assert!(stored.messages[1].continuation.is_none());
}
