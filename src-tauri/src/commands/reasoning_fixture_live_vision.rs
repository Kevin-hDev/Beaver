use super::runner::TurnEvidence;
use super::support::LiveSpec;
use crate::services::agent_local::types_session::PreserveReasoningSetting;
use serde::Serialize;

#[derive(Serialize)]
struct VisionReport {
    schema_version: u8,
    source: &'static str,
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
    session.preserve_reasoning = if spec.provider == "ollama" {
        PreserveReasoningSetting::Local
    } else {
        PreserveReasoningSetting::Remote
    };
    crate::services::agent_local::session_store::save(&session).await?;

    let attachment = super::super::reasoning_fixture_vision::inline_attachment()?;
    let encoded = super::super::reasoning_fixture_vision::inline_base64()?;
    let image_turn = super::runner::run_turn(
        app,
        spec,
        &session.id,
        "Inspect the attached four-quadrant image. In this same turn use fixture.write_note with value fixture, then fixture.read_note. Reply only with the four quadrant colors in reading order, separated by commas.",
        vec![attachment],
    )
    .await?;
    super::evidence::require_tool_round(&session.id, &image_turn).await?;
    require_color_answer(&session.id, &image_turn).await?;

    let recall_turn = super::runner::run_turn(
        app,
        spec,
        &session.id,
        "Without a new attachment or tool, what color is the top-left quadrant? Reply with the color only.",
        Vec::new(),
    )
    .await?;
    require_top_left_answer(&session.id, &recall_turn).await?;
    validate_history(&session.id, &encoded, &image_turn, &recall_turn).await?;
    write_report(&session.id, spec, &image_turn, &recall_turn).await
}

async fn require_color_answer(session_id: &str, turn: &TurnEvidence) -> Result<(), String> {
    let session = crate::services::agent_local::session_store::get(session_id).await?;
    let content = session
        .messages
        .iter()
        .filter(|message| message.role == "assistant" && message.turn_id == turn.turn_id)
        .map(|message| message.content.as_str())
        .collect::<String>()
        .to_ascii_lowercase();
    super::evidence::answer_matches(&content, "red green blue yellow")
        .then_some(())
        .ok_or_else(unavailable)
}

async fn require_top_left_answer(session_id: &str, turn: &TurnEvidence) -> Result<(), String> {
    super::evidence::require_recall_without_tools(session_id, turn, "red").await
}

async fn validate_history(
    session_id: &str,
    encoded: &str,
    image_turn: &TurnEvidence,
    recall_turn: &TurnEvidence,
) -> Result<(), String> {
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
    for request_id in [&image_turn.request_id, &recall_turn.request_id] {
        let run = session
            .diagnostic_runs
            .iter()
            .find(|run| &run.request_id == request_id)
            .ok_or_else(unavailable)?;
        if run.status != "completed"
            || run
                .events
                .iter()
                .filter(|event| event.phase == "provider_payload")
                .count()
                == 0
        {
            return Err(unavailable());
        }
    }
    Ok(())
}

async fn write_report(
    session_id: &str,
    spec: &LiveSpec,
    image_turn: &TurnEvidence,
    recall_turn: &TurnEvidence,
) -> Result<(), String> {
    let generated_at = chrono::Utc::now();
    let fixture_id = crate::services::reasoning_fixture_store::derive_fixture_id_with_variant(
        spec.provider,
        spec.model,
        &format!("vision-{}", spec.mode),
        spec.region,
        generated_at.date_naive(),
    )
    .map_err(|_| unavailable())?;
    let scenarios = vec![
        scenario(
            "image_input_and_response",
            &[image_turn],
            "vision decision=\"image_accepted\" count=1",
        )?,
        scenario(
            "history_image_continuity",
            &[recall_turn],
            "vision decision=\"history_reused\" new_images=0",
        )?,
        scenario(
            "diagnostics_redacted",
            &[image_turn, recall_turn],
            "vision decision=\"diagnostics_redacted\"",
        )?,
    ];
    let report = VisionReport {
        schema_version: 1,
        source: "runner",
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
    turns: &[&TurnEvidence],
    decision: &'static str,
) -> Result<VisionScenario, String> {
    let run_id = turns
        .last()
        .map(|turn| turn.request_id.clone())
        .ok_or_else(unavailable)?;
    let request_count = turns.iter().map(|turn| turn.payload_count).sum();
    if request_count == 0 {
        return Err(unavailable());
    }
    Ok(VisionScenario {
        requirement,
        run_id,
        status: "passe",
        request_count,
        reasoning_event_count: turns.iter().map(|turn| turn.reasoning_event_count).sum(),
        decisions: vec![decision],
    })
}

fn unavailable() -> String {
    "vision fixture failed".to_string()
}
