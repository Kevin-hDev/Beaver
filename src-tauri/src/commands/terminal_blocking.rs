pub async fn run<T, Operation>(operation: Operation) -> Result<T, String>
where
    T: Send + 'static,
    Operation: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|_| "terminal-error".to_string())?
}
