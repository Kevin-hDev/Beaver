use super::storage_load::ForecastLoadError;
use super::types::{ForecastAnalysisMeta, ForecastResult, ForecastWorkspace};
use tokio::sync::Mutex;

static LEGACY_CLAIM_LOCK: Mutex<()> = Mutex::const_new(());

pub async fn save_for_session(session_id: &str, result: &mut ForecastResult) -> Result<(), String> {
    let workspace = crate::services::workspace_scope::resolve(session_id).await?;
    match &result.workspace {
        ForecastWorkspace::Legacy => {
            if let Some(owner_session_id) = result.session_id.as_deref() {
                let owner = crate::services::workspace_scope::resolve(owner_session_id).await?;
                if owner != workspace {
                    return Err(access_error());
                }
            }
            result.workspace = workspace;
        }
        current if current != &workspace => return Err(access_error()),
        _ => {}
    }
    if result.session_id.is_none() {
        result.session_id = Some(session_id.to_string());
    }
    super::storage::save(result).await
}

pub async fn list_for_session(session_id: &str) -> Result<Vec<ForecastAnalysisMeta>, String> {
    let workspace = crate::services::workspace_scope::resolve(session_id).await?;
    let entries = super::storage_index::list().await?;
    let mut visible = Vec::new();
    for mut entry in entries {
        if entry.workspace == workspace {
            visible.push(entry);
            continue;
        }
        if entry.workspace != ForecastWorkspace::Legacy {
            continue;
        }
        let Some(owner_session_id) = entry.session_id.as_deref() else {
            continue;
        };
        if crate::services::workspace_scope::resolve(owner_session_id).await? != workspace {
            continue;
        }
        let mut analysis = super::storage::load(&entry.id).await?;
        analysis.workspace = workspace.clone();
        super::storage::save(&mut analysis).await?;
        entry = analysis.to_meta();
        visible.push(entry);
    }
    Ok(visible)
}

pub async fn list_unassigned_for_session(
    session_id: &str,
) -> Result<Vec<ForecastAnalysisMeta>, String> {
    crate::services::workspace_scope::resolve(session_id).await?;
    Ok(super::storage_index::list()
        .await?
        .into_iter()
        .filter(|entry| entry.workspace == ForecastWorkspace::Legacy && entry.session_id.is_none())
        .collect())
}

pub async fn claim_legacy_for_session(
    session_id: &str,
    id: &str,
) -> Result<ForecastResult, String> {
    // La revendication doit être indivisible : deux fenêtres ne peuvent pas
    // attribuer la même analyse héritée à deux espaces différents.
    let _claim_guard = LEGACY_CLAIM_LOCK.lock().await;
    let mut analysis = super::storage::load(id).await?;
    if analysis.workspace != ForecastWorkspace::Legacy || analysis.session_id.is_some() {
        return Err(access_error());
    }
    save_for_session(session_id, &mut analysis).await?;
    Ok(analysis)
}

pub async fn load_for_session(session_id: &str, id: &str) -> Result<ForecastResult, String> {
    load_classified_for_session(session_id, id)
        .await
        .map_err(|error| error.message().to_string())
}

pub async fn load_classified_for_session(
    session_id: &str,
    id: &str,
) -> Result<ForecastResult, ForecastLoadError> {
    let workspace = crate::services::workspace_scope::resolve(session_id)
        .await
        .map_err(|_| ForecastLoadError::Unavailable)?;
    let mut analysis = super::storage::load_classified(id).await?;
    if analysis.workspace == ForecastWorkspace::Legacy {
        let owner_session_id = analysis
            .session_id
            .as_deref()
            .ok_or(ForecastLoadError::NotFound)?;
        let legacy_workspace = crate::services::workspace_scope::resolve(owner_session_id)
            .await
            .map_err(|_| ForecastLoadError::Unavailable)?;
        if legacy_workspace != workspace {
            return Err(ForecastLoadError::NotFound);
        }
        analysis.workspace = workspace;
        super::storage::save(&mut analysis)
            .await
            .map_err(|_| ForecastLoadError::Unavailable)?;
        return Ok(analysis);
    }
    (analysis.workspace == workspace)
        .then_some(analysis)
        .ok_or(ForecastLoadError::NotFound)
}

pub async fn authorize_for_session(session_id: &str, id: &str) -> Result<(), String> {
    load_for_session(session_id, id).await.map(|_| ())
}

pub async fn delete_for_session(session_id: &str, id: &str) -> Result<(), String> {
    authorize_for_session(session_id, id).await?;
    super::storage::delete(id).await
}

pub async fn rename_for_session(
    session_id: &str,
    id: &str,
    name: &str,
) -> Result<ForecastResult, String> {
    let mut analysis = load_for_session(session_id, id).await?;
    analysis.name = super::storage_paths::validate_analysis_name(name)?;
    save_for_session(session_id, &mut analysis).await?;
    Ok(analysis)
}

fn access_error() -> String {
    "Analyse introuvable".into()
}
