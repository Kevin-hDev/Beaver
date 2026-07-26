use crate::services::agent_local::types_ollama::ChatMessage;
use crate::ActiveStreams;

#[tauri::command]
pub async fn queue_agent_message(
    session_id: String,
    generation: u64,
    messages: Vec<ChatMessage>,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<bool, String> {
    crate::services::agent_local::session_store::validate_session_id(&session_id)
        .map_err(|_| generic_error())?;
    let inbox = {
        let map = streams.0.lock().await;
        let Some((_, active_generation, _, inbox)) = map.get(&session_id) else {
            return Ok(false);
        };
        if generation != *active_generation {
            return Ok(false);
        }
        inbox.clone()
    };
    inbox.enqueue(messages).await.map_err(|_| generic_error())
}

fn generic_error() -> String {
    "Impossible d'envoyer ce message".to_string()
}
