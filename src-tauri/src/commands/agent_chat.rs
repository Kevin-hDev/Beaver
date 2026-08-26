#![expect(
    clippy::too_many_arguments,
    reason = "Tauri chat boundary keeps provider options explicit"
)]

use crate::models::agent_turn_contract::{ChatStreamAdmission, TurnStart};
use crate::ActiveStreams;

#[tauri::command]
pub async fn chat_stream(
    app: tauri::AppHandle,
    session_id: String,
    model: String,
    turn: TurnStart,
    tools: Vec<serde_json::Value>,
    think: bool,
    provider: Option<String>,
    working_dir: Option<String>,
    supports_tools: Option<bool>,
    supports_thinking: Option<bool>,
    supports_vision: Option<bool>,
    reasoning_mode: Option<String>,
    permission_mode: Option<String>,
    plan_mode: Option<bool>,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<ChatStreamAdmission, String> {
    super::agent_chat_run::start(
        app,
        super::agent_chat_run::ChatStreamRequest {
            session_id,
            model,
            turn: Some(turn),
            tools,
            think,
            provider: provider.unwrap_or_else(|| "ollama".to_string()),
            working_dir,
            capability_hints: super::agent_chat_task::StreamCapabilityHints {
                supports_tools,
                supports_thinking,
                supports_vision,
            },
            reasoning_mode,
            permission_mode,
            plan_mode,
        },
        &streams,
    )
    .await
}
