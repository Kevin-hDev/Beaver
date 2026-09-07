use crate::models::agent_turn_contract::{ChatStreamRequestInput, NewUserTurnInput, TurnStart};
use crate::services::agent_local::types_session::AgentSession;
use crate::ActiveStreams;
use std::time::{Duration, Instant};
use tauri::Manager;

const TURN_TIMEOUT: Duration = Duration::from_secs(240);
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(5);

pub(super) struct TurnEvidence {
    pub(super) turn_id: String,
    pub(super) request_id: String,
    pub(super) payload_count: usize,
    pub(super) reasoning_event_count: usize,
}

pub(super) async fn run_turn(
    app: &tauri::App,
    spec: &super::support::LiveSpec,
    session_id: &str,
    content: &str,
    files: Vec<crate::models::agent_turn_contract::TurnAttachmentInput>,
) -> Result<TurnEvidence, String> {
    let request =
        crate::commands::agent_chat_run::ChatStreamRequest::from_input(ChatStreamRequestInput {
            session_id: session_id.to_string(),
            model: spec.model.to_string(),
            provider: spec.provider.to_string(),
            turn: TurnStart::New(NewUserTurnInput {
                content: content.to_string(),
                files,
                skills: Vec::new(),
            }),
            working_dir: None,
            permission_mode: Some("auto".to_string()),
            plan_mode: Some(false),
        });
    let admission = crate::commands::agent_chat_run::start_fixture(
        app.handle().clone(),
        request,
        &app.state::<ActiveStreams>(),
    )
    .await?;
    wait_for_turn(app, session_id, admission.generation, admission.turn_id).await
}

async fn wait_for_turn(
    app: &tauri::App,
    session_id: &str,
    generation: u64,
    turn_id: String,
) -> Result<TurnEvidence, String> {
    let deadline = Instant::now() + TURN_TIMEOUT;
    loop {
        let active_generation = app
            .state::<ActiveStreams>()
            .0
            .lock()
            .await
            .get(session_id)
            .map(|(_, active, _, _)| *active);
        match active_generation {
            Some(active) if active != generation => return Err("fixture request replaced".into()),
            Some(_) if Instant::now() >= deadline => {
                cancel_timed_out_turn(app, session_id, generation).await?;
                return Err("fixture timeout".into());
            }
            Some(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            None => {
                let session = wait_for_terminal_session(session_id, generation).await?;
                return evidence(session, generation, turn_id);
            }
        }
    }
}

async fn cancel_timed_out_turn(
    app: &tauri::App,
    session_id: &str,
    generation: u64,
) -> Result<(), String> {
    crate::commands::agent_chat_cancel::cancel_agent_request(
        app.handle().clone(),
        session_id.to_string(),
        Some(generation),
        app.state::<ActiveStreams>(),
    )
    .await
    .map_err(|_| "fixture timeout cleanup failed".to_string())?;
    let deadline = Instant::now() + CLEANUP_TIMEOUT;
    loop {
        let still_active = app
            .state::<ActiveStreams>()
            .0
            .lock()
            .await
            .get(session_id)
            .is_some_and(|(_, active, _, _)| *active == generation);
        if !still_active {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("fixture timeout cleanup failed".into());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn wait_for_terminal_session(
    session_id: &str,
    generation: u64,
) -> Result<AgentSession, String> {
    let deadline = Instant::now() + CLEANUP_TIMEOUT;
    loop {
        let session = crate::services::agent_local::session_store::get(session_id).await?;
        if let Some(run) = session
            .diagnostic_runs
            .iter()
            .find(|run| run.generation == generation)
        {
            if run.ended_at.is_some() {
                return Ok(session);
            }
        }
        if Instant::now() >= deadline {
            return Err("fixture request failed".into());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn evidence(
    session: AgentSession,
    generation: u64,
    turn_id: String,
) -> Result<TurnEvidence, String> {
    let run = session
        .diagnostic_runs
        .iter()
        .find(|run| run.generation == generation)
        .ok_or_else(|| "fixture request failed".to_string())?;
    let payload_count = run
        .events
        .iter()
        .filter(|event| event.phase == "provider_payload")
        .count();
    if run.status != "completed" || payload_count == 0 {
        return Err("fixture request failed".into());
    }
    Ok(TurnEvidence {
        turn_id,
        request_id: run.request_id.clone(),
        payload_count,
        reasoning_event_count: run
            .events
            .iter()
            .filter(|event| event.phase == "reasoning")
            .count(),
    })
}
