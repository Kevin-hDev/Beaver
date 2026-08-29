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
    crate::services::api_keys::init_for_runtime()
        .expect("initialize configured credentials for this runtime");
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
        run_turn(
            app,
            spec,
            &session.id,
            "Call fixture.read_note, then report its value briefly.",
        )
        .await?;
        super::reasoning_fixture::export_reasoning_fixture_report_with_variant(
            session.id.clone(),
            spec.region.to_string(),
            (spec.provider == "deepseek").then_some(spec.mode),
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

#[cfg(test)]
mod vision_tests {
    #[test]
    fn synthetic_png_is_small_deterministic_and_has_four_exact_quadrants() {
        let first = super::super::reasoning_fixture_vision::png_bytes().unwrap();
        let second = super::super::reasoning_fixture_vision::png_bytes().unwrap();
        assert_eq!(first, second);
        assert!(first.len() < 4 * 1024);
        assert_eq!(super::super::reasoning_fixture_vision::MIME, "image/png");
        assert!(!super::super::reasoning_fixture_vision::inline_base64()
            .unwrap()
            .is_empty());

        let image = image::load_from_memory(&first).unwrap().to_rgba8();
        assert_eq!(image.dimensions(), (8, 8));
        assert_eq!(image.get_pixel(1, 1).0, [255, 0, 0, 255]);
        assert_eq!(image.get_pixel(6, 1).0, [0, 255, 0, 255]);
        assert_eq!(image.get_pixel(1, 6).0, [0, 0, 255, 255]);
        assert_eq!(image.get_pixel(6, 6).0, [255, 255, 0, 255]);
    }
}
