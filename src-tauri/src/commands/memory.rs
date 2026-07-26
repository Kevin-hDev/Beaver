use crate::services::agent_local::memory_types::{
    MemoryMode, MemoryOverview, MemoryScopeOverview, MemorySettings,
};

#[tauri::command]
pub async fn get_memory_overview(
    working_dir: Option<String>,
    session_id: Option<String>,
) -> Result<MemoryOverview, String> {
    let resolved = resolve_working_dir(working_dir.as_deref(), session_id.as_deref()).await?;
    Ok(crate::services::agent_local::memory_overview::load(resolved.as_deref()).await)
}

#[tauri::command]
pub async fn get_memory_project_topics(project_id: String) -> Result<MemoryScopeOverview, String> {
    crate::services::agent_local::memory_overview::load_project(&project_id).await
}

#[tauri::command]
pub async fn set_memory_mode(mode: String) -> Result<MemorySettings, String> {
    let parsed = match mode.as_str() {
        "disabled" => MemoryMode::Disabled,
        "manual" => MemoryMode::Manual,
        "automatic" => MemoryMode::Automatic,
        _ => return Err("Mode mémoire invalide.".into()),
    };
    crate::services::agent_local::memory_settings::set_mode(parsed).await
}

#[tauri::command]
pub async fn set_memory_context_budget(tokens: u32) -> Result<MemorySettings, String> {
    if !(256..=3_000).contains(&tokens) {
        return Err("Budget mémoire invalide.".into());
    }
    crate::services::agent_local::memory_settings::set_budget(tokens).await
}

#[tauri::command]
pub async fn archive_memory_topic(
    path: String,
    session_id: Option<String>,
) -> Result<MemoryOverview, String> {
    let layout = crate::services::agent_local::memory_paths::MemoryLayout::production();
    let (scope, topic_path) = layout.management_topic(&path)?;
    crate::services::agent_local::memory_store::archive_topic(&scope, &topic_path).await?;
    let working_dir = resolve_working_dir(None, session_id.as_deref()).await?;
    Ok(crate::services::agent_local::memory_overview::load(working_dir.as_deref()).await)
}

async fn resolve_working_dir(
    working_dir: Option<&str>,
    session_id: Option<&str>,
) -> Result<Option<std::path::PathBuf>, String> {
    let candidate = match working_dir.filter(|value| !value.is_empty()) {
        Some(path) => Some(path.to_string()),
        None => match session_id.filter(|value| !value.is_empty()) {
            Some(id) => crate::services::agent_local::session_store::get(id)
                .await
                .ok()
                .map(|session| session.working_dir)
                .filter(|path| !path.is_empty()),
            None => None,
        },
    };
    candidate
        .map(|path| {
            std::path::PathBuf::from(path)
                .canonicalize()
                .map_err(|_| "Projet mémoire inaccessible.".to_string())
        })
        .transpose()
}
