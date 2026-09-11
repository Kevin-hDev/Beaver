use super::*;
use crate::services::agent_local::context_usage_record::{
    ContextCountCoverage, ContextCountSource, ContextTokenCount,
};

#[tokio::test]
async fn unverified_local_count_is_not_persisted_as_in_flight() {
    let session = super::super::session_store::create_full(
        "Unknown local context",
        "fixture",
        "ollama",
        false,
        None,
    )
    .await
    .unwrap();
    let journal = super::super::conversation_journal::ConversationJournal::new(
        session.id.clone(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
    )
    .unwrap();
    journal.activate_context_request().await.unwrap();
    let emitter = super::super::stream_events::AgentEventEmitter::test(session.id.clone());
    let attempt = super::super::context_usage_runtime::PreparedContextAttempt::new(
        super::super::context_usage_runtime::ContextAttempt {
            on_event: &emitter,
            journal: Some(&journal),
            provider_id: "ollama",
            model: "fixture",
            turn: 0,
            attempt: 1,
            context_limit: 32_000,
            measured_input_source: ContextCountSource::NativeCounter,
        },
        Default::default(),
    );
    let unknown = ContextTokenCount {
        tokens: None,
        capacity_tokens: None,
        source: None,
        coverage: ContextCountCoverage::Unknown,
    };

    assert_eq!(
        persist_verified_context(&unknown, Some(&attempt))
            .await
            .unwrap_err(),
        super::super::context_capacity_error::UNVERIFIED_CODE
    );
    let saved = super::super::session_store::get(&session.id).await.unwrap();
    assert!(saved.context_usage.current_preparation.is_none());
    super::super::session_store::delete_one(&session.id)
        .await
        .unwrap();
}
