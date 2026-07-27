pub fn emit_started(session_id: &str, mode: &str) {
    let payload = serde_json::json!({
        "sessionId": session_id,
        "mode": mode,
    });
    tauri::async_runtime::spawn(async move {
        crate::services::extensions::emit_event("session.turn.started", payload).await;
    });
}
