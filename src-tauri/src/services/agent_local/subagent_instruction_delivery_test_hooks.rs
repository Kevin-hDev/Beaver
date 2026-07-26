pub(super) async fn drain_with_before_save<F, Fut>(
    session_id: &str,
    messages: &mut Vec<ChatMessage>,
    before_save: F,
) -> Result<usize, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    drain_inner(session_id, messages, || async {}, before_save).await
}

pub(super) async fn drain_with_after_registry_read<F, Fut>(
    session_id: &str,
    messages: &mut Vec<ChatMessage>,
    after_registry_read: F,
) -> Result<usize, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    drain_inner(session_id, messages, after_registry_read, || async {}).await
}
