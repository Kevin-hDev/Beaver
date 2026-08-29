use crate::models::agent_turn_contract::{ChatStreamRequestInput, NewUserTurnInput, TurnStart};
use crate::ActiveStreams;
use std::time::{Duration, Instant};
use tauri::Manager;

#[path = "reasoning_fixture_live_support.rs"]
mod support;
#[path = "reasoning_fixture_live_vision.rs"]
mod vision;

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
    let vision_specs = LIVE_SPECS
        .iter()
        .filter(|spec| {
            matches!(spec.provider, "anthropic" | "qwen")
                && spec.mode == "medium"
                && selected.as_deref().is_none_or(|routes| {
                    routes.split(',').any(|route| route.trim() == spec.provider)
                })
                && !failures
                    .iter()
                    .any(|failure| failure.starts_with(&format!("{}:", spec.provider)))
        })
        .collect::<Vec<_>>();
    for spec in vision_specs {
        if let Err(error) = vision::run(app, spec).await {
            failures.push(format!("{}:vision={error}", spec.provider));
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
            Vec::new(),
        )
        .await?;
        run_turn(
            app,
            spec,
            &session.id,
            "Call fixture.read_note, then report its value briefly.",
            Vec::new(),
        )
        .await?;
        super::reasoning_fixture::export_reasoning_fixture_report_with_variant(
            session.id.clone(),
            spec.region.to_string(),
            matches!(spec.provider, "deepseek" | "anthropic" | "qwen").then_some(spec.mode),
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
    files: Vec<crate::models::agent_turn_contract::TurnAttachmentInput>,
) -> Result<(), String> {
    let request = super::agent_chat_run::ChatStreamRequest::from_input(ChatStreamRequestInput {
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
        let attachment = super::super::reasoning_fixture_vision::inline_attachment().unwrap();
        assert!(attachment.path.is_empty());
        assert_eq!(attachment.mime_type, "image/png");
        assert_eq!(attachment.size, first.len() as u64);
        assert!(attachment
            .thumbnail
            .as_deref()
            .unwrap()
            .starts_with("data:image/png;base64,"));

        let image = image::load_from_memory(&first).unwrap().to_rgba8();
        assert_eq!(image.dimensions(), (64, 64));
        assert_eq!(image.get_pixel(1, 1).0, [255, 0, 0, 255]);
        assert_eq!(image.get_pixel(62, 1).0, [0, 255, 0, 255]);
        assert_eq!(image.get_pixel(1, 62).0, [0, 0, 255, 255]);
        assert_eq!(image.get_pixel(62, 62).0, [255, 255, 0, 255]);
    }

    #[test]
    fn live_specs_include_each_validated_provider_mode_once() {
        let tuples = super::LIVE_SPECS
            .iter()
            .filter(|spec| matches!(spec.provider, "anthropic" | "qwen"))
            .map(|spec| (spec.provider, spec.model, spec.region, spec.mode))
            .collect::<Vec<_>>();
        assert_eq!(
            tuples,
            vec![
                ("anthropic", "claude-haiku-4-5-20251001", "france", "low"),
                ("anthropic", "claude-haiku-4-5-20251001", "france", "medium"),
                ("anthropic", "claude-haiku-4-5-20251001", "france", "high"),
                ("qwen", "qwen3.8-flash", "singapore", "low"),
                ("qwen", "qwen3.8-flash", "singapore", "medium"),
                ("qwen", "qwen3.8-flash", "singapore", "xhigh"),
            ]
        );
    }

    #[test]
    fn checked_in_vision_reports_prove_both_live_routes_without_payload_data() {
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test-fixtures/vision-reports");
        for fixture_id in [
            "anthropic-api-claude-haiku-4-5-20251001-vision-medium-france-2026-08-29",
            "qwen-api-qwen3-8-flash-vision-medium-singapore-2026-08-29",
        ] {
            let bytes = std::fs::read(root.join(format!("{fixture_id}.json")))
                .expect("checked-in vision proof");
            assert!(bytes.len() <= 64 * 1024);
            let report: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(report["fixture_id"], fixture_id);
            let scenarios = report["scenarios"].as_array().unwrap();
            for requirement in [
                "image_input_and_response",
                "history_image_continuity",
                "diagnostics_redacted",
            ] {
                assert!(scenarios.iter().any(|scenario| {
                    scenario["requirement"] == requirement && scenario["status"] == "passe"
                }));
            }
            let text = String::from_utf8(bytes).unwrap();
            assert!(!text.contains("data:image"));
            assert!(!text.contains("base64"));
            assert!(!text.contains("VISION_OK"));
            assert!(!text.contains("RED"));
        }
    }
}
