use super::support::LiveSpec;
use crate::services::agent_local::types_session::PreserveReasoningSetting;
use serde::Serialize;

#[derive(Serialize)]
struct VisionReport {
    schema_version: u8,
    fixture_id: String,
    route: String,
    model: String,
    region: String,
    reasoning_mode: String,
    generated_at: String,
    scenarios: Vec<VisionScenario>,
}

#[derive(Serialize)]
struct VisionScenario {
    requirement: &'static str,
    run_id: String,
    status: &'static str,
    request_count: usize,
    reasoning_event_count: usize,
    decisions: Vec<&'static str>,
}

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
    let request_ids = validate_history(&session.id, &encoded).await?;
    write_report(&session.id, spec, &request_ids).await
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

async fn validate_history(session_id: &str, encoded: &str) -> Result<[String; 2], String> {
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
    let request_ids = session
        .diagnostic_runs
        .iter()
        .filter(|run| {
            run.status == "completed"
                && run
                    .events
                    .iter()
                    .any(|event| event.phase == "provider_payload")
        })
        .map(|run| run.request_id.clone())
        .collect::<Vec<_>>();
    request_ids.try_into().map_err(|_| unavailable())
}

async fn write_report(
    session_id: &str,
    spec: &LiveSpec,
    request_ids: &[String; 2],
) -> Result<(), String> {
    let generated_at = chrono::Utc::now();
    let fixture_id = crate::services::reasoning_fixture_store::derive_fixture_id_with_variant(
        spec.provider,
        spec.model,
        "vision-medium",
        spec.region,
        generated_at.date_naive(),
    )
    .map_err(|_| unavailable())?;
    let scenarios = vec![
        scenario(
            "image_input_and_response",
            request_ids[0].clone(),
            1,
            "vision decision=\"image_accepted\" count=1",
        ),
        scenario(
            "history_image_continuity",
            request_ids[1].clone(),
            1,
            "vision decision=\"history_reused\" new_images=0",
        ),
        scenario(
            "diagnostics_redacted",
            request_ids[1].clone(),
            2,
            "vision decision=\"diagnostics_redacted\"",
        ),
    ];
    let report = VisionReport {
        schema_version: 1,
        fixture_id: fixture_id.clone(),
        route: spec.provider.to_string(),
        model: spec.model.to_string(),
        region: spec.region.to_string(),
        reasoning_mode: spec.mode.to_string(),
        generated_at: generated_at.to_rfc3339(),
        scenarios,
    };
    let bytes = serde_json::to_vec(&report).map_err(|_| unavailable())?;
    crate::services::reasoning_fixture_store::write_report(session_id, &fixture_id, bytes)
        .await
        .map_err(|_| unavailable())
}

fn scenario(
    requirement: &'static str,
    run_id: String,
    request_count: usize,
    decision: &'static str,
) -> VisionScenario {
    VisionScenario {
        requirement,
        run_id,
        status: "passe",
        request_count,
        reasoning_event_count: 0,
        decisions: vec![decision],
    }
}

fn unavailable() -> String {
    "vision fixture failed".to_string()
}
