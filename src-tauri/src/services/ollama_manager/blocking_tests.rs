use super::blocking::run_ollama_blocking;
use super::error::OllamaErrorCode;
use std::sync::mpsc;
use std::time::Duration;

#[tokio::test]
async fn blocking_operation_does_not_stop_async_progress() {
    let (release_tx, release_rx) = mpsc::channel();
    let operation = tokio::spawn(run_ollama_blocking(move || {
        release_rx
            .recv()
            .map_err(|_| OllamaErrorCode::OllamaInternal)?;
        Ok::<_, OllamaErrorCode>(())
    }));
    tokio::time::timeout(
        Duration::from_secs(1),
        tokio::time::sleep(Duration::from_millis(10)),
    )
    .await
    .expect("timer must progress while blocking work waits");
    release_tx.send(()).unwrap();
    operation.await.unwrap().unwrap();
}

#[tokio::test]
async fn join_failure_is_mapped_to_internal_error() {
    let result = run_ollama_blocking(|| -> Result<(), OllamaErrorCode> {
        panic!("test panic");
    })
    .await;
    assert_eq!(result, Err(OllamaErrorCode::OllamaInternal));
}
