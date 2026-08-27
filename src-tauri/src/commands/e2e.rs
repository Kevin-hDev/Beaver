use crate::models::agent_turn_contract::{ChatStreamRequestInput, NewUserTurnInput, TurnStart};
use crate::services::agent_local::session_store;
use crate::services::agent_local::types_session::AgentSession;
use crate::ActiveStreams;
use serde::Serialize;
use std::time::Duration;
use tauri::Manager;

const COORDINATED_EXIT_DELAY: Duration = Duration::from_secs(1);

#[tauri::command]
pub fn e2e_request_exit(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(COORDINATED_EXIT_DELAY).await;
        crate::app_exit::request(&app, 0);
    });
}

#[tauri::command]
pub fn e2e_native_webviews() -> crate::services::browser::process_role::NativeWebViewObservation {
    crate::services::browser::observe_native_webviews()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct E2eChildReadOnlyOutcome {
    code: String,
    session_unchanged: bool,
    request_start_unchanged: bool,
    active_stream_absent: bool,
}

#[tauri::command]
pub async fn e2e_verify_child_chat_stream_read_only(
    app: tauri::AppHandle,
    streams: tauri::State<'_, ActiveStreams>,
) -> Result<E2eChildReadOnlyOutcome, String> {
    let mut session = session_store::create_full("E2E child", "e2e-model", "ollama", false, None)
        .await
        .map_err(|_| fixture_error())?;
    session.parent_session_id = Some(uuid::Uuid::new_v4().to_string());
    if session_store::save(&session).await.is_err() {
        let _ = session_store::delete_one(&session.id).await;
        return Err(fixture_error());
    }

    let before_session = serialized_session(&session);
    let before_request_starts = request_start_count(&session);
    let absent_before = !streams.0.lock().await.contains_key(&session.id);
    let result = super::agent_chat::chat_stream_from_input(
        app.clone(),
        ChatStreamRequestInput {
            session_id: session.id.clone(),
            model: "e2e-model".to_string(),
            provider: "ollama".to_string(),
            turn: TurnStart::New(NewUserTurnInput {
                content: "read-only boundary".to_string(),
                files: Vec::new(),
                skills: Vec::new(),
            }),
            working_dir: None,
            permission_mode: None,
            plan_mode: None,
        },
        &streams,
    )
    .await;
    let after_session = session_store::get(&session.id).await;
    let absent_after = !app
        .state::<ActiveStreams>()
        .0
        .lock()
        .await
        .contains_key(&session.id);
    let cleanup = session_store::delete_one(&session.id).await;

    let before_session = before_session?;
    let after_session = after_session.map_err(|_| fixture_error())?;
    cleanup.map_err(|_| fixture_error())?;
    let code = result
        .err()
        .filter(|error| {
            error == crate::services::agent_local::session_user_write::SUBAGENT_READ_ONLY
        })
        .unwrap_or_else(|| "unexpected-outcome".to_string());

    Ok(E2eChildReadOnlyOutcome {
        code,
        session_unchanged: serialized_session(&after_session)? == before_session,
        request_start_unchanged: request_start_count(&after_session) == before_request_starts,
        active_stream_absent: absent_before && absent_after,
    })
}

fn serialized_session(session: &AgentSession) -> Result<serde_json::Value, String> {
    serde_json::to_value(session).map_err(|_| fixture_error())
}

fn request_start_count(session: &AgentSession) -> usize {
    session
        .diagnostic_runs
        .iter()
        .flat_map(|run| &run.events)
        .filter(|event| event.phase == "request_start")
        .count()
}

fn fixture_error() -> String {
    "E2E fixture unavailable".to_string()
}
