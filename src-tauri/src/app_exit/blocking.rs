pub(super) async fn execute<Operation, Output>(operation: Operation) -> Output
where
    Operation: FnOnce() -> Output + Send + 'static,
    Output: Send + 'static,
{
    match tokio::task::spawn_blocking(operation).await {
        Ok(output) => output,
        Err(error) if error.is_panic() => std::panic::resume_unwind(error.into_panic()),
        Err(_) => panic!("blocking cleanup task cancelled"),
    }
}
