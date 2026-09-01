use super::types::ForecastWorkspace;
use tokio::sync::OnceCell;

static ORPHAN_REPAIR: OnceCell<()> = OnceCell::const_new();

pub(super) async fn ensure_orphaned_owners_repaired() -> Result<(), String> {
    // Les suppressions nominales passent par le guichet lifecycle ; seuls des
    // restes d'une interruption antérieure peuvent exister au démarrage.
    ORPHAN_REPAIR
        .get_or_try_init(repair_orphaned_owners)
        .await
        .map(|_| ())
}

async fn repair_orphaned_owners() -> Result<(), String> {
    let _guard = super::workspace_lifecycle::lock().await;
    let snapshot = crate::services::workspace_scope::WorkspaceSnapshot::load().await?;
    let entries = super::storage_index::list().await?;
    let workspaces = entries
        .iter()
        .filter(|entry| {
            entry.workspace != ForecastWorkspace::Legacy && !snapshot.is_live(&entry.workspace)
        })
        .map(|entry| entry.workspace.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let deleted_sessions = entries
        .iter()
        .filter(|entry| entry.workspace == ForecastWorkspace::Legacy)
        .filter_map(|entry| entry.session_id.as_ref())
        .filter(|session_id| !snapshot.contains_session(session_id))
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if workspaces.is_empty() && deleted_sessions.is_empty() {
        return Ok(());
    }
    super::storage_scope::release_workspaces_locked(&workspaces, &deleted_sessions).await
}
