use crate::services::workspace_scope::WorkspaceScope;
use tokio::sync::{Mutex, MutexGuard};

static LIFECYCLE_LOCK: Mutex<()> = Mutex::const_new(());

// Cycle de vie unique : supprimer un projet ou une discussion racine libère
// analyses et profils vers Legacy avant le propriétaire; les notes suivent
// l'analyse. Les anciens profils sont revendiqués une seule fois à la lecture.
// Les onglets terminal restent côté UI et sont retirés seulement après succès.

pub(crate) async fn lock() -> MutexGuard<'static, ()> {
    LIFECYCLE_LOCK.lock().await
}

pub(crate) async fn release_session_family_locked(
    workspace: Option<&WorkspaceScope>,
    session_ids: &[String],
) -> Result<(), String> {
    super::storage_scope::release_workspace_locked(workspace, session_ids).await
}

pub async fn delete_project(project_id: &str) -> Result<(), String> {
    crate::services::agent_local::project_store::validate_project_id(project_id)?;
    let _guard = lock().await;
    let session_ids = crate::services::agent_local::session_index::read_index()
        .await?
        .into_iter()
        .filter(|session| session.project_id.as_deref() == Some(project_id))
        .map(|session| session.id)
        .collect::<Vec<_>>();
    super::storage_scope::release_workspace_locked(
        Some(&WorkspaceScope::Project(project_id.to_string())),
        &session_ids,
    )
    .await?;
    crate::services::agent_local::session_store::clear_project_id(project_id).await?;
    // Le projet disparaît en dernier : chaque étape préparatoire est durable et
    // idempotente, donc une interruption laisse toujours une suppression rejouable.
    crate::services::agent_local::project_store::delete(project_id).await
}
