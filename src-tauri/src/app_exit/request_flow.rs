use super::final_action::{self, FinalActionSource};
use super::{cleanup, policy, presentation, raw_exit, AppExitCoordinator, BeginResult, ExitIntent};
use tauri::Manager;

pub(super) fn handle_requested(
    app: &tauri::AppHandle,
    code: Option<i32>,
    api: &tauri::ExitRequestApi,
) {
    ::log::info!("[exit] coordinated shutdown requested");
    let (intent, exit_code) = requested_intent(code);
    let coordinator = app.state::<AppExitCoordinator>();
    match coordinator.begin_with_intent(
        intent,
        exit_code,
        crate::services::browser::begin_cef_shutdown,
    ) {
        BeginResult::Ready => {}
        BeginResult::Waiting => api.prevent_exit(),
        BeginResult::InvariantViolation => raw_exit::terminate_process(1),
        BeginResult::Started(timeline, owned_intent) => {
            api.prevent_exit();
            start_cleanup(app, &coordinator, timeline, owned_intent, exit_code);
        }
    }
}

pub(super) fn requested_intent(code: Option<i32>) -> (ExitIntent, i32) {
    if code == Some(super::BEAVER_RESTART_REQUEST_CODE) {
        (ExitIntent::Restart, 0)
    } else {
        (ExitIntent::Exit, code.unwrap_or_default())
    }
}

fn start_cleanup(
    app: &tauri::AppHandle,
    coordinator: &AppExitCoordinator,
    timeline: policy::ShutdownTimeline,
    intent: ExitIntent,
    exit_code: i32,
) {
    if let Some(manager) = app.try_state::<crate::services::ollama_manager::OllamaManager>() {
        manager.begin_closing();
    }
    if coordinator
        .spawn_watchdog(app.clone(), timeline, intent, exit_code)
        .is_err()
    {
        ::log::error!("[exit] watchdog unavailable; ultimate guard remains armed");
    }
    presentation::hide_application(app);
    let handle = app.clone();
    let registry = coordinator.registry.clone();
    tauri::async_runtime::spawn(async move {
        let started = std::time::Instant::now();
        if !registry
            .wait_empty_until(timeline.graceful_deadline())
            .await
        {
            ::log::warn!("[exit] tracked work exceeded graceful deadline");
        }
        log_cleanup_outcome(cleanup::run(&handle, timeline).await);
        ::log::info!("[exit] cleanup phase finished in {:?}", started.elapsed());
        let coordinator = handle.state::<AppExitCoordinator>();
        final_action::run(
            &coordinator.state,
            intent,
            exit_code,
            FinalActionSource::Cleanup,
            |intent, code| final_action::dispatch_tauri(&handle, intent, code),
        );
    });
}

fn log_cleanup_outcome(outcome: cleanup::CleanupOutcome) {
    match outcome {
        cleanup::CleanupOutcome::Completed => {}
        cleanup::CleanupOutcome::TimedOut => ::log::warn!("[exit] graceful deadline reached"),
        cleanup::CleanupOutcome::Panicked => ::log::warn!("[exit] cleanup interrupted"),
    }
}
