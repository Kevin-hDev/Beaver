use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForecastWorkbenchContext {
    pub session_id: String,
    pub analysis_id: Option<String>,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ForecastWorkbenchSnapshot {
    pub context: ForecastWorkbenchContext,
    pub draft: super::workbench_drafts::ForecastWorkbenchDraft,
    pub analysis_name: Option<String>,
}

#[derive(Clone)]
struct ActiveWorkbench {
    context: ForecastWorkbenchContext,
    draft: super::workbench_drafts::ForecastWorkbenchDraft,
}

static ACTIVE_WORKBENCH: Mutex<Option<ActiveWorkbench>> = Mutex::const_new(None);
const MAX_CONTEXT_NAME_CHARS: usize = 120;

pub async fn set(
    session_id: String,
    analysis_id: Option<String>,
) -> Result<ForecastWorkbenchSnapshot, String> {
    validate_ids(&session_id, analysis_id.as_deref())?;
    crate::services::agent_local::session_store::get(&session_id)
        .await
        .map_err(|_| context_error())?;
    let analysis = load_analysis(analysis_id.as_deref()).await?;
    let mut active = ACTIVE_WORKBENCH.lock().await;
    if let Some(current) = active.as_ref() {
        super::workbench_drafts::save(&current.context, &current.draft).await?;
    }
    let context = next_context(
        active.as_ref().map(|current| &current.context),
        session_id,
        analysis_id,
    );
    let draft = super::workbench_drafts::load(&context).await?;
    *active = Some(ActiveWorkbench {
        context: context.clone(),
        draft: draft.clone(),
    });
    Ok(ForecastWorkbenchSnapshot {
        context,
        draft,
        analysis_name: analysis.map(|value| bounded_name(value.name)),
    })
}

pub async fn get() -> Result<Option<ForecastWorkbenchSnapshot>, String> {
    let active = ACTIVE_WORKBENCH.lock().await.clone();
    let Some(active) = active else {
        return Ok(None);
    };
    crate::services::agent_local::session_store::get(&active.context.session_id)
        .await
        .map_err(|_| context_error())?;
    let analysis = load_analysis(active.context.analysis_id.as_deref()).await?;
    Ok(Some(ForecastWorkbenchSnapshot {
        context: active.context,
        draft: active.draft,
        analysis_name: analysis.map(|value| bounded_name(value.name)),
    }))
}

pub async fn update_draft(
    section: String,
) -> Result<super::workbench_drafts::ForecastWorkbenchDraft, String> {
    let mut active = ACTIVE_WORKBENCH.lock().await;
    let current = active.as_ref().ok_or_else(context_error)?;
    let next = super::workbench_drafts::next(&current.draft, section)?;
    super::workbench_drafts::save(&current.context, &next).await?;
    let current = active.as_mut().ok_or_else(context_error)?;
    current.draft = next.clone();
    Ok(next)
}

async fn load_analysis(
    analysis_id: Option<&str>,
) -> Result<Option<super::types::ForecastResult>, String> {
    let Some(analysis_id) = analysis_id else {
        return Ok(None);
    };
    // L'historique est global : le chat actif peut consulter une analyse créée ailleurs.
    let analysis = super::storage::load(analysis_id)
        .await
        .map_err(|_| context_error())?;
    Ok(Some(analysis))
}

fn next_context(
    current: Option<&ForecastWorkbenchContext>,
    session_id: String,
    analysis_id: Option<String>,
) -> ForecastWorkbenchContext {
    ForecastWorkbenchContext {
        session_id,
        analysis_id,
        revision: current
            .and_then(|value| value.revision.checked_add(1))
            .unwrap_or(1),
    }
}

fn validate_ids(session_id: &str, analysis_id: Option<&str>) -> Result<(), String> {
    crate::services::agent_local::session_store::validate_session_id(session_id)
        .map_err(|_| context_error())?;
    if let Some(id) = analysis_id {
        uuid::Uuid::parse_str(id).map_err(|_| context_error())?;
    }
    Ok(())
}

fn bounded_name(value: String) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(MAX_CONTEXT_NAME_CHARS)
        .collect()
}

fn context_error() -> String {
    "Contexte Forecast indisponible".to_string()
}

#[cfg(test)]
#[path = "workbench_context_tests.rs"]
mod tests;
