use super::*;
use std::sync::LazyLock;
use tokio::sync::Mutex;

static PENDING_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[tokio::test]
async fn pending_store_is_bounded() {
    let _guard = PENDING_TEST_LOCK.lock().await;
    fill_pending_for_test(64).await;

    assert_eq!(pending_len_for_test().await, 64);
    clear_pending_for_test().await;
}

#[tokio::test]
async fn wrong_session_cannot_resolve_a_choice() {
    let _guard = PENDING_TEST_LOCK.lock().await;
    clear_pending_for_test().await;
    insert_pending_for_test("choice-1", "session-a").await;

    let result = respond("session-b", "choice-1", vec![]).await;

    assert!(result.is_err());
    assert_eq!(pending_len_for_test().await, 1);
    clear_pending_for_test().await;
}

#[tokio::test]
async fn invalid_response_keeps_the_choice_available() {
    let _guard = PENDING_TEST_LOCK.lock().await;
    clear_pending_for_test().await;
    let _receiver = insert_pending_receiver_for_test("choice-1", "session-a").await;
    let invalid = vec![AgentInteractiveAnswer {
        question_index: 0,
        selected_ids: vec!["unexpected".into()],
        selected_labels: vec![],
        custom_answer: None,
    }];

    let result = respond("session-a", "choice-1", invalid).await;

    assert!(result.is_err());
    assert_eq!(pending_len_for_test().await, 1);
    clear_pending_for_test().await;
}

#[tokio::test]
async fn dismiss_resolves_without_fabricating_answers() {
    let _guard = PENDING_TEST_LOCK.lock().await;
    clear_pending_for_test().await;
    let receiver = insert_pending_receiver_for_test("choice-1", "session-a").await;

    dismiss("session-a", "choice-1").await.unwrap();

    assert_eq!(receiver.await.unwrap(), InteractiveChoiceResponse::Dismissed);
    assert_eq!(pending_len_for_test().await, 0);
}
