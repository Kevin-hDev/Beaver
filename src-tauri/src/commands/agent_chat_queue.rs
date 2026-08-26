use crate::models::agent_turn_contract::NewUserTurnInput;
use crate::ActiveStreams;

#[tauri::command]
pub async fn queue_agent_message(
    session_id: String,
    generation: u64,
    input: NewUserTurnInput,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<bool, String> {
    crate::services::agent_local::session_store::validate_session_id(&session_id)
        .map_err(|_| generic_error())?;
    crate::services::agent_local::session_user_write::ensure_allowed(&session_id).await?;
    crate::services::agent_local::conversation_input::validate_intention(&input)
        .map_err(|_| generic_error())?;
    let map = streams.0.lock().await;
    let current = matches!(
        map.get(&session_id),
        Some((_, active_generation, _, _)) if generation == *active_generation
    );
    drop(map);
    if !current {
        return Ok(false);
    }
    // Aucun consommateur durable n'est encore branché : refuser honnêtement
    // conserve le brouillon côté UI au lieu de promettre une file fantôme.
    Ok(false)
}

fn generic_error() -> String {
    "Impossible d'envoyer ce message".to_string()
}
