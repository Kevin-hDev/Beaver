use crate::models::agent_turn_contract::{ChatStreamAdmission, ChatStreamRequestInput};
use crate::ActiveStreams;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChatStreamInvokeInput {
    request: ChatStreamRequestInput,
}

pub(crate) fn decode_chat_stream_request(
    body: &tauri::ipc::InvokeBody,
) -> Result<ChatStreamRequestInput, String> {
    let value = match body {
        tauri::ipc::InvokeBody::Json(value) => value.clone(),
        tauri::ipc::InvokeBody::Raw(_) => return Err("conversation_admission_failed".to_string()),
    };
    serde_json::from_value::<ChatStreamInvokeInput>(value)
        .map(|root| root.request)
        .map_err(|_| "conversation_admission_failed".to_string())
}

#[tauri::command]
pub async fn chat_stream(
    app: tauri::AppHandle,
    ipc_request: tauri::ipc::Request<'_>,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<ChatStreamAdmission, String> {
    let request = decode_chat_stream_request(ipc_request.body())?;
    super::agent_chat_run::start(
        app,
        super::agent_chat_run::ChatStreamRequest::from_input(request),
        &streams,
    )
    .await
}
