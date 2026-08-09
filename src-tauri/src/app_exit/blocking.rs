pub(super) async fn execute<Operation, Output>(
    operation: Operation,
) -> Result<Output, tokio::task::JoinError>
where
    Operation: FnOnce() -> Output + Send + 'static,
    Output: Send + 'static,
{
    tokio::task::spawn_blocking(operation).await
}
