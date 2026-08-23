use super::types_session::AgentSession;

pub(super) async fn update_locked_with<R, F, A, AFut, W, WFut>(
    id: &str,
    mutate: F,
    after_load: A,
    writer: W,
) -> Result<R, String>
where
    F: FnOnce(&mut AgentSession) -> R,
    A: FnOnce() -> AFut,
    AFut: std::future::Future<Output = ()>,
    W: FnOnce(AgentSession) -> WFut,
    WFut: std::future::Future<Output = Result<(), String>>,
{
    super::session_store::validate_session_id(id)?;
    let lock = super::session_store::lock_session(id).await;
    let _guard = lock.lock().await;
    let mut session = super::session_store::get(id).await?;
    after_load().await;
    let result = mutate(&mut session);
    writer(session).await?;
    Ok(result)
}

pub(super) async fn update_locked<R>(
    id: &str,
    mutate: impl FnOnce(&mut AgentSession) -> R,
) -> Result<R, String> {
    update_locked_with(
        id,
        mutate,
        || async {},
        |session| async move { super::session_store::save(&session).await },
    )
    .await
}

#[cfg(test)]
pub(super) async fn update_fast_mode_with_writer<W, Fut>(
    id: &str,
    enabled: bool,
    writer: W,
) -> Result<bool, String>
where
    W: FnOnce(AgentSession) -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    update_locked_with(
        id,
        |session| {
            session.fast_mode_enabled = enabled;
            session.fast_mode_enabled
        },
        || async {},
        writer,
    )
    .await
}

#[cfg(test)]
pub(super) async fn update_fast_mode_with_after_load<F, Fut>(
    id: &str,
    enabled: bool,
    after_load: F,
) -> Result<bool, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    update_locked_with(
        id,
        |session| {
            session.fast_mode_enabled = enabled;
            session.fast_mode_enabled
        },
        after_load,
        |session| async move { super::session_store::save(&session).await },
    )
    .await
}

#[cfg(test)]
pub(super) async fn assign_project_with_after_load<F, Fut>(
    id: &str,
    project_id: &str,
    after_load: F,
) -> Result<bool, String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    update_locked_with(
        id,
        |session| {
            if session.project_id.is_some() {
                return false;
            }
            session.project_id = Some(project_id.to_string());
            true
        },
        after_load,
        |session| async move { super::session_store::save(&session).await },
    )
    .await
}
