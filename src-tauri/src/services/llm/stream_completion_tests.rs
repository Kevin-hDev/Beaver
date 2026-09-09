use super::*;

#[test]
fn thinking_only_is_not_a_success_but_a_real_tool_call_is() {
    let mut result = StreamResult {
        thinking: "private reasoning".into(),
        ..Default::default()
    };
    finish(&mut result);
    assert_eq!(result.completion_error, Some("provider_empty_response"));
    result
        .tool_calls
        .push(("read".into(), serde_json::json!({})));
    finish(&mut result);
    assert_eq!(result.completion_error, None);
}

#[test]
fn truncated_and_filtered_outputs_cannot_be_used_as_summaries() {
    for (reason, expected) in [
        ("length", "provider_output_limit"),
        ("content_filter", "provider_content_filtered"),
    ] {
        let mut result = StreamResult {
            content: "partial answer".into(),
            done_reason: Some(reason.into()),
            ..Default::default()
        };
        finish(&mut result);
        assert_eq!(require_complete(result).unwrap_err(), expected);
    }
}

#[test]
fn native_results_are_not_reclassified_without_the_chat_reader() {
    let result = StreamResult {
        done_reason: Some("max_tokens".into()),
        ..Default::default()
    };
    assert!(require_complete(result).is_ok());
}

#[tokio::test]
async fn failed_answer_guard_returns_the_actionable_error() {
    let mut result = StreamResult {
        done_reason: Some("length".into()),
        content: "partial answer".into(),
        ..Default::default()
    };
    finish(&mut result);
    let error = reject_if_failed(
        &AgentEventEmitter::test("failed-answer".into()),
        &result,
        false,
        None,
        739,
        8192,
    )
    .await
    .unwrap_err();
    assert_eq!(error, "provider_output_limit");
}

#[tokio::test]
async fn partial_answer_survives_reload_without_committing_a_successful_turn() {
    use crate::services::agent_local::session_store;
    let session =
        session_store::create_full("Partial completion", "fixture", "openrouter", false, None)
            .await
            .unwrap();
    let mut journal = ConversationJournal::new(
        session.id.clone(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
    )
    .unwrap();
    let mut result = StreamResult {
        content: "partial answer".into(),
        done_reason: Some("length".into()),
        ..Default::default()
    };
    finish(&mut result);
    let error = reject_if_failed(
        &AgentEventEmitter::test(session.id.clone()),
        &result,
        false,
        Some(&mut journal),
        739,
        8192,
    )
    .await
    .unwrap_err();
    let saved = session_store::get(&session.id).await.unwrap();
    let commit = journal.commit_turn().await;
    session_store::delete_one(&session.id).await.unwrap();
    assert_eq!(error, "provider_output_limit");
    assert_eq!(saved.messages.last().unwrap().content, "partial answer");
    assert!(saved.messages.last().unwrap().tool_calls.is_none());
    assert!(commit.is_err());
}
