use super::error::OllamaErrorCode;
use super::retry::{OllamaRecoveryRetry, RetryCategory, RetryWait, OLLAMA_RECOVERY_RETRY_DELAYS};
use std::time::Duration;

#[test]
fn retry_schedule_is_five_fifteen_sixty_three_hundred_then_saturated() {
    let mut retry = OllamaRecoveryRetry::new();
    let expected = [
        Duration::from_secs(5),
        Duration::from_secs(15),
        Duration::from_secs(60),
        Duration::from_secs(300),
        Duration::from_secs(300),
    ];
    assert_eq!(OLLAMA_RECOVERY_RETRY_DELAYS, expected[..4]);
    for delay in expected {
        assert_eq!(retry.next_delay(), delay);
    }
}

#[test]
fn durable_progress_resets_retry_sequence() {
    let mut retry = OllamaRecoveryRetry::new();
    assert_eq!(retry.next_delay(), Duration::from_secs(5));
    assert_eq!(retry.next_delay(), Duration::from_secs(15));
    retry.reset_after_progress();
    assert_eq!(retry.next_delay(), Duration::from_secs(5));
}

#[test]
fn one_timer_and_closing_are_explicit() {
    let retry = OllamaRecoveryRetry::new();
    assert_eq!(retry.begin_timer(), Some(Duration::from_secs(5)));
    assert_eq!(retry.begin_timer(), None);
    retry.finish_timer();
    retry.close();
    assert_eq!(retry.begin_timer(), None);
    assert_eq!(retry.request_wake(), Err(OllamaErrorCode::OllamaClosing));
}

#[test]
fn retry_logs_are_deduplicated_by_code_and_category() {
    let retry = OllamaRecoveryRetry::new();
    let code = OllamaErrorCode::OllamaRecoveryDeferred;
    assert!(retry.should_log(code, RetryCategory::Recovery));
    assert!(!retry.should_log(code, RetryCategory::Recovery));
    assert!(retry.should_log(code, RetryCategory::Validation));
    assert!(retry.should_log(code, RetryCategory::Storage));
}

#[tokio::test(start_paused = true)]
async fn manual_wake_reuses_the_single_timer() {
    let retry = OllamaRecoveryRetry::new();
    let cancellation = tokio_util::sync::CancellationToken::new();
    let task_retry = retry.clone();
    let task_cancel = cancellation.clone();
    let task = tokio::spawn(async move { task_retry.wait(&task_cancel).await });
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(4)).await;
    assert!(!task.is_finished());
    retry.request_wake().unwrap();
    assert_eq!(task.await.unwrap(), RetryWait::Due);
}
