use crate::models::agent_turn_contract::{ChatStreamRequestInput, NewUserTurnInput, TurnStart};
use crate::ActiveStreams;
use std::time::{Duration, Instant};
use tauri::Manager;

#[path = "reasoning_fixture_live_support.rs"]
mod support;

use support::{LiveSpec, LIVE_SPECS};

pub(crate) async fn refresh_live_reasoning_fixture_matrix_once(
    app: &tauri::App,
) -> Result<(), String> {
    crate::services::api_keys::init().expect("initialize configured credentials");
    let selected = std::env::var("BEAVER_FIXTURE_ROUTES").ok();
    if selected
        .as_deref()
        .is_none_or(|routes| routes.split(',').any(|route| route.trim() == "ollama"))
    {
        support::prepare_ollama(app).await?;
    }
    let mut failures = Vec::new();

    for spec in LIVE_SPECS.iter().filter(|spec| {
        selected
            .as_deref()
            .is_none_or(|routes| routes.split(',').any(|route| route.trim() == spec.provider))
    }) {
        if let Err(error) = run_spec(app, spec).await {
            failures.push(format!("{}:{}={error}", spec.provider, spec.model));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

async fn run_spec(app: &tauri::App, spec: &LiveSpec) -> Result<(), String> {
    let mut session = crate::services::agent_local::session_store::create_full(
        "Reasoning fixture",
        spec.model,
        spec.provider,
        false,
        None,
    )
    .await?;
    session.reasoning_mode = Some(spec.mode.to_string());
    session.thinking_enabled = true;
    session.preserve_reasoning = if spec.provider == "ollama" {
        crate::services::agent_local::types_session::PreserveReasoningSetting::Local
    } else {
        crate::services::agent_local::types_session::PreserveReasoningSetting::Remote
    };
    crate::services::agent_local::session_store::save(&session).await?;
    let result = async {
        run_turn(
            app,
            spec,
            &session.id,
            "Call fixture.write_note with value fixture, then confirm briefly.",
        )
        .await?;
        if spec.provider != "deepseek" {
            run_turn(
                app,
                spec,
                &session.id,
                "Call fixture.read_note, then report its value briefly.",
            )
            .await?;
        }
        super::reasoning_fixture::export_reasoning_fixture_report(
            session.id.clone(),
            spec.region.to_string(),
        )
        .await
        .map(|_| ())
    }
    .await;
    if result.is_err() {
        let diagnostic = crate::services::agent_local::session_store::get(&session.id)
            .await
            .ok()
            .and_then(|stored| {
                stored
                    .stream_failures
                    .last()
                    .map(|failure| failure.code.clone())
                    .or_else(|| {
                        stored
                            .diagnostic_runs
                            .last()
                            .and_then(|run| run.safe_summary.clone())
                    })
            })
            .unwrap_or_else(|| "fixture failed".to_string());
        return Err(diagnostic);
    }
    Ok(())
}

async fn run_turn(
    app: &tauri::App,
    spec: &LiveSpec,
    session_id: &str,
    content: &str,
) -> Result<(), String> {
    let request = super::agent_chat_run::ChatStreamRequest::from_input(ChatStreamRequestInput {
        session_id: session_id.to_string(),
        model: spec.model.to_string(),
        provider: spec.provider.to_string(),
        turn: TurnStart::New(NewUserTurnInput {
            content: content.to_string(),
            files: Vec::new(),
            skills: Vec::new(),
        }),
        working_dir: None,
        permission_mode: Some("auto".to_string()),
        plan_mode: Some(false),
    });
    super::agent_chat_run::start_fixture(
        app.handle().clone(),
        request,
        &app.state::<ActiveStreams>(),
    )
    .await?;
    let deadline = Instant::now() + Duration::from_secs(240);
    loop {
        if !app
            .state::<ActiveStreams>()
            .0
            .lock()
            .await
            .contains_key(session_id)
        {
            break;
        }
        if Instant::now() >= deadline {
            return Err("fixture timeout".to_string());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let session = crate::services::agent_local::session_store::get(session_id).await?;
    session
        .diagnostic_runs
        .last()
        .filter(|run| run.status == "completed")
        .map(|_| ())
        .ok_or_else(|| "fixture request failed".to_string())
}
