use crate::models::agent_turn_contract::{ChatStreamAdmission, ChatStreamRequestInput};
use crate::ActiveStreams;

#[tauri::command]
pub async fn chat_stream(
    app: tauri::AppHandle,
    request: ChatStreamRequestInput,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<ChatStreamAdmission, String> {
    super::agent_chat_run::start(
        app,
        super::agent_chat_run::ChatStreamRequest {
            session_id: request.session_id,
            model: request.model,
            turn: Some(request.turn),
            tools: Vec::new(),
            think: false,
            provider: request.provider,
            working_dir: request.working_dir,
            capability_hints: Default::default(),
            reasoning_mode: None,
            permission_mode: request.permission_mode,
            plan_mode: request.plan_mode,
        },
        &streams,
    )
    .await
}
