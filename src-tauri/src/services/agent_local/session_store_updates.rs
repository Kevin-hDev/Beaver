use super::session_store::{get, save, validate_session_id};
#[cfg(test)]
pub(super) use super::session_store_update_gate::{
    update_fast_mode_with_after_load, update_fast_mode_with_writer,
};

pub(super) async fn update_locked<R>(
    id: &str,
    mutate: impl FnOnce(&mut super::types_session::AgentSession) -> R,
) -> Result<R, String> {
    super::session_store_update_gate::update_locked(id, mutate).await
}

pub async fn update_fast_mode(id: &str, enabled: bool) -> Result<bool, String> {
    update_locked(id, |session| {
        session.fast_mode_enabled = enabled;
        session.fast_mode_enabled
    })
    .await
}

pub async fn update_model(
    id: &str,
    model: &str,
    provider: &str,
    reasoning_mode: Option<String>,
    supports_thinking: Option<bool>,
) -> Result<(), String> {
    update_locked(id, |session| {
        let previous_mode = reasoning_mode.or_else(|| session.reasoning_mode.clone());
        let supports_thinking = supports_thinking.unwrap_or_else(|| {
            crate::services::reasoning::provider_model_supports_thinking(provider, model)
        });
        session.model = model.to_string();
        session.provider = provider.to_string();
        session.reasoning_mode = crate::services::reasoning::normalize_for_model(
            provider,
            model,
            previous_mode.as_deref(),
            supports_thinking,
        );
        session.thinking_enabled =
            crate::services::reasoning::enabled(session.reasoning_mode.as_deref(), false);
        session.context_tokens = None;
    })
    .await
}

pub async fn update_reasoning(
    id: &str,
    reasoning_mode: Option<String>,
    supports_thinking: Option<bool>,
) -> Result<(), String> {
    update_locked(id, |session| {
        let mode = crate::services::reasoning::sanitize_mode(reasoning_mode);
        let supports_thinking = supports_thinking.unwrap_or_else(|| {
            if session.provider == "ollama" && mode.is_some() {
                true
            } else {
                crate::services::reasoning::provider_model_supports_thinking(
                    &session.provider,
                    &session.model,
                )
            }
        });
        let mode = crate::services::reasoning::normalize_for_model(
            &session.provider,
            &session.model,
            mode.as_deref(),
            supports_thinking,
        );
        session.thinking_enabled = !matches!(mode.as_deref(), None | Some("off"));
        session.reasoning_mode = mode;
        session.context_tokens = None;
    })
    .await
}

pub async fn update_working_dir(id: &str, dir: &str) -> Result<(), String> {
    update_working_dir_inner(
        id,
        dir,
        ManagedAssignment::Set(false),
        ProjectAssignment::Preserve,
        || async {},
    )
    .await
}

pub async fn set_managed_working_dir(id: &str, dir: &str) -> Result<(), String> {
    update_working_dir_inner(
        id,
        dir,
        ManagedAssignment::Set(true),
        ProjectAssignment::Preserve,
        || async {},
    )
    .await
}

pub async fn refresh_working_dir(id: &str, dir: &str) -> Result<(), String> {
    update_working_dir_inner(
        id,
        dir,
        ManagedAssignment::Preserve,
        ProjectAssignment::Preserve,
        || async {},
    )
    .await
}

pub async fn switch_working_dir_to_project(
    id: &str,
    dir: &str,
    project_id: &str,
) -> Result<(), String> {
    update_working_dir_inner(
        id,
        dir,
        ManagedAssignment::Set(false),
        ProjectAssignment::Set(project_id),
        || async {},
    )
    .await
}

enum ProjectAssignment<'a> {
    Preserve,
    Set(&'a str),
}

enum ManagedAssignment {
    Preserve,
    Set(bool),
}

async fn update_working_dir_inner<F, Fut>(
    id: &str,
    dir: &str,
    managed: ManagedAssignment,
    project: ProjectAssignment<'_>,
    after_load: F,
) -> Result<(), String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    validate_session_id(id)?;
    let path = std::path::Path::new(dir);
    if !path.is_absolute() || !path.is_dir() {
        return Err(format!("Répertoire invalide : {dir}"));
    }
    let canonical = dunce::canonicalize(path).map_err(|e| format!("Canonicalize : {e}"))?;
    let lock = super::session_store::lock_session(id).await;
    let _guard = lock.lock().await;
    let mut session = get(id).await?;
    after_load().await;
    let managed_after_update = match managed {
        ManagedAssignment::Set(value) => value,
        ManagedAssignment::Preserve => session.working_dir_managed,
    };
    if !managed_after_update {
        super::directory_access::ensure_allowed(&canonical)?;
    }
    session.working_dir = canonical.to_string_lossy().to_string();
    session.context_tokens = None;
    if let ManagedAssignment::Set(value) = managed {
        session.working_dir_managed = value;
    }
    match project {
        ProjectAssignment::Preserve => {}
        ProjectAssignment::Set(project_id) => session.project_id = Some(project_id.to_string()),
    }
    save(&session).await
}

#[cfg(test)]
pub(super) async fn update_working_dir_with_after_load<F, Fut>(
    id: &str,
    dir: &str,
    after_load: F,
) -> Result<(), String>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    update_working_dir_inner(
        id,
        dir,
        ManagedAssignment::Set(false),
        ProjectAssignment::Preserve,
        after_load,
    )
    .await
}
