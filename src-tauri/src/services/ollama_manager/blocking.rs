#![allow(dead_code)]

use super::error::OllamaErrorCode;

pub(super) async fn run_ollama_blocking<T, F>(operation: F) -> Result<T, OllamaErrorCode>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, OllamaErrorCode> + Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|_| OllamaErrorCode::OllamaInternal)?
}
