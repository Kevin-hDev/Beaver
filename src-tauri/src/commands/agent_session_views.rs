use crate::models::agent_session_contract::AgentSessionView;
use crate::services::agent_local::session_store;

#[tauri::command]
pub async fn get_agent_session(id: String) -> Result<AgentSessionView, String> {
    let session = session_store::get(&id).await?;
    crate::services::agent_local::session_view::from_session(&session)
}

#[tauri::command]
pub async fn create_agent_session(
    name: String,
    model: String,
    provider: Option<String>,
    project_id: Option<String>,
    reasoning_mode: Option<String>,
    supports_thinking: Option<bool>,
    fast_mode_enabled: Option<bool>,
) -> Result<AgentSessionView, String> {
    let provider = provider.unwrap_or_else(|| "ollama".to_string());
    let mut session = session_store::create_with_project_and_fast_mode(
        &name,
        &model,
        &provider,
        project_id,
        fast_mode_enabled.unwrap_or(false),
    )
    .await?;
    if reasoning_mode.is_some() {
        session_store::update_reasoning(&session.id, reasoning_mode, supports_thinking).await?;
        if let Ok(updated) = session_store::get(&session.id).await {
            session = updated;
        }
    }
    crate::services::agent_local::session_view::from_session(&session)
}
