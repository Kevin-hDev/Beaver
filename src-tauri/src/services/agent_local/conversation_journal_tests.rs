use super::conversation_journal::{validate_tool_results, ConversationJournal};
use super::session_store;
use super::types_ollama::ChatMessage;

#[test]
fn journal_rejects_missing_duplicate_and_reordered_tool_results() {
    let expected = vec!["call-a".to_string(), "call-b".to_string()];
    assert!(validate_tool_results(&[tool("call-a"), tool("call-b")], &expected).is_ok());
    assert!(validate_tool_results(&[tool("call-a")], &expected).is_err());
    assert!(validate_tool_results(&[tool("call-a"), tool("call-a")], &expected).is_err());
    assert!(validate_tool_results(&[tool("call-b"), tool("call-a")], &expected).is_err());
}

#[tokio::test]
async fn partial_checkpoint_never_commits_a_turn_as_final() {
    let session = session_store::create_full("Partial journal", "model", "groq", false, None)
        .await
        .expect("create session");
    let mut journal = ConversationJournal::new(
        session.id.clone(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
        uuid::Uuid::new_v4().to_string(),
    )
    .expect("create journal");
    journal
        .persist_assistant_step(&ChatMessage::assistant(
            "complete step".into(),
            None,
            None,
            None,
            None,
        ))
        .await
        .expect("persist complete step");
    journal
        .persist_partial(ChatMessage::assistant(
            "interrupted step".into(),
            None,
            None,
            None,
            None,
        ))
        .await
        .expect("persist partial step");

    assert!(journal.commit_turn().await.is_err());
    session_store::delete_one(&session.id)
        .await
        .expect("delete session");
}

fn tool(id: &str) -> ChatMessage { ChatMessage::tool("result".into(), Some(id.into()), Some("bash".into())) }
