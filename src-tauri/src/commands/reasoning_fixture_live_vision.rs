use super::support::LiveSpec;
use crate::services::agent_local::types_session::PreserveReasoningSetting;

pub(super) async fn run(app: &tauri::App, spec: &LiveSpec) -> Result<(), String> {
    let mut session = crate::services::agent_local::session_store::create_full(
        "Vision fixture",
        spec.model,
        spec.provider,
        false,
        None,
    )
    .await?;
    session.reasoning_mode = Some(spec.mode.to_string());
    session.thinking_enabled = true;
    session.preserve_reasoning = PreserveReasoningSetting::Remote;
    crate::services::agent_local::session_store::save(&session).await?;

    let attachment = super::super::reasoning_fixture_vision::inline_attachment()?;
    let encoded = super::super::reasoning_fixture_vision::inline_base64()?;
    super::run_turn(
        app,
        spec,
        &session.id,
        "Inspect the attached four-quadrant image and reply exactly VISION_OK.",
        vec![attachment],
    )
    .await?;
    require_last_assistant(&session.id, "VISION_OK").await?;

    super::run_turn(
        app,
        spec,
        &session.id,
        "Without a new attachment, reply exactly RED for the top-left quadrant.",
        Vec::new(),
    )
    .await?;
    require_last_assistant(&session.id, "RED").await?;
    validate_history(&session.id, &encoded).await
}

async fn require_last_assistant(session_id: &str, marker: &str) -> Result<(), String> {
    let session = crate::services::agent_local::session_store::get(session_id).await?;
    let content = session
        .messages
        .iter()
        .rev()
        .find(|message| message.role == "assistant")
        .map(|message| message.content.trim())
        .ok_or_else(unavailable)?;
    content
        .contains(marker)
        .then_some(())
        .ok_or_else(unavailable)
}

async fn validate_history(session_id: &str, encoded: &str) -> Result<(), String> {
    let session = crate::services::agent_local::session_store::get(session_id).await?;
    let images = session
        .messages
        .iter()
        .flat_map(|message| &message.files)
        .filter(|file| file.mime_type == super::super::reasoning_fixture_vision::MIME)
        .count();
    if images != 1 {
        return Err(unavailable());
    }
    let diagnostics = serde_json::to_string(&session.diagnostic_runs).map_err(|_| unavailable())?;
    if diagnostics.contains(encoded) || diagnostics.contains("data:image/png;base64,") {
        return Err(unavailable());
    }
    Ok(())
}

fn unavailable() -> String {
    "vision fixture failed".to_string()
}
