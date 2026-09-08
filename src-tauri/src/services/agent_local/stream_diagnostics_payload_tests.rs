use super::*;
use crate::services::agent_local::{session_store, stream_diagnostics};

#[tokio::test]
async fn payload_summary_is_bounded_like_its_persisted_event() {
    let session = session_store::create_full("Payload summary", "test", "ollama", false, None)
        .await
        .unwrap();
    let request_id = stream_diagnostics::start_request(&session.id, 1).await;
    // Large counters exercise clipping without changing the numeric-only payload contract.
    let stats = PayloadStats {
        items: usize::MAX,
        assistant_items: usize::MAX,
        reasoning_chars: usize::MAX,
        instructions_chars: usize::MAX,
        ..Default::default()
    };
    record_payload(&session.id, &request_id, 0, "ollama", "ollama_chat", stats).await;
    let stored = session_store::get(&session.id).await.unwrap();
    session_store::delete_one(&session.id).await.unwrap();
    let run = stored.diagnostic_runs.last().unwrap();
    let summary = run.safe_summary.as_deref().unwrap();
    assert_eq!(run.phase, "provider_payload");
    assert!(summary.starts_with("provider_payload provider=ollama"));
    assert!(summary.ends_with("..."));
    assert!(summary.chars().count() <= 203);
    assert_eq!(summary, run.events.last().unwrap().message);
}
