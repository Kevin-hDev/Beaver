use super::*;
use crate::services::agent_local::{session_store, stream_diagnostics};
use serde_json::json;

#[tokio::test]
async fn final_payload_is_recorded_once_without_its_content() {
    let session = session_store::create_full("Payload summary", "test", "ollama", false, None)
        .await
        .unwrap();
    let request_id = stream_diagnostics::start_request(&session.id, 1).await;
    let sample_content = "never-persist-this-content";
    let messages: Vec<_> = (0..50)
        .map(|index| {
            json!({
                "role": "tool",
                "content": format!("{sample_content}-{index}"),
            })
        })
        .collect();
    let payload = json!({"messages": messages, "tools": [{"type": "function"}]});

    record_provider_payload(
        Some(&session.id),
        Some(&request_id),
        "ollama",
        "ollama_chat",
        &payload,
    )
    .await;

    let stored = session_store::get(&session.id).await.unwrap();
    session_store::delete_one(&session.id).await.unwrap();
    let run = stored.diagnostic_runs.last().unwrap();
    let events: Vec<_> = run
        .events
        .iter()
        .filter(|event| event.phase == "provider_payload")
        .collect();
    assert_eq!(events.len(), 1);
    assert!(events[0].message.contains("tool_results=50"));
    assert!(!events[0].message.contains(sample_content));
    assert_eq!(run.safe_summary.as_deref(), Some(events[0].message.as_str()));
}

#[tokio::test]
async fn payload_summary_remains_bounded() {
    let session = session_store::create_full("Payload summary", "test", "ollama", false, None)
        .await
        .unwrap();
    let request_id = stream_diagnostics::start_request(&session.id, 1).await;
    let provider = "p".repeat(300);

    record_provider_payload(
        Some(&session.id),
        Some(&request_id),
        &provider,
        "ollama_chat",
        &json!({"messages": []}),
    )
    .await;

    let stored = session_store::get(&session.id).await.unwrap();
    session_store::delete_one(&session.id).await.unwrap();
    let summary = stored
        .diagnostic_runs
        .last()
        .unwrap()
        .safe_summary
        .as_deref()
        .unwrap();
    assert!(summary.ends_with("..."));
    assert!(summary.chars().count() <= 203);
}
