use crate::services::gateway::GatewayService;
use crate::{services, ActiveStreams};
use futures_util::FutureExt;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Duration;
use tauri::{ExitRequestApi, Manager};

const PHASE_IDLE: u8 = 0;
const PHASE_CLEANING: u8 = 1;
const PHASE_READY: u8 = 2;
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Default)]
pub struct AppExitCoordinator {
    phase: AtomicU8,
}

#[derive(Debug, PartialEq, Eq)]
enum BeginResult {
    Started,
    Waiting,
    Ready,
}

impl AppExitCoordinator {
    fn begin(&self) -> BeginResult {
        match self.phase.compare_exchange(
            PHASE_IDLE,
            PHASE_CLEANING,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => BeginResult::Started,
            Err(PHASE_READY) => BeginResult::Ready,
            Err(_) => BeginResult::Waiting,
        }
    }

    fn mark_ready(&self) {
        self.phase.store(PHASE_READY, Ordering::Release);
    }
}

pub fn request(app: &tauri::AppHandle, code: i32) {
    hide_application(app);
    app.exit(code);
}

pub fn handle_requested(app: &tauri::AppHandle, code: Option<i32>, api: &ExitRequestApi) {
    if code == Some(tauri::RESTART_EXIT_CODE) {
        return;
    }
    match app.state::<AppExitCoordinator>().begin() {
        BeginResult::Ready => {}
        BeginResult::Waiting => api.prevent_exit(),
        BeginResult::Started => {
            api.prevent_exit();
            hide_application(app);
            let handle = app.clone();
            tauri::async_runtime::spawn(async move {
                let started = std::time::Instant::now();
                let cleanup =
                    std::panic::AssertUnwindSafe(cleanup_services(&handle)).catch_unwind();
                match tokio::time::timeout(CLEANUP_TIMEOUT, cleanup).await {
                    Err(_) => eprintln!("[exit] délai global atteint, fermeture forcée"),
                    Ok(Err(_)) => eprintln!("[exit] nettoyage interrompu, fermeture forcée"),
                    Ok(Ok(())) => {}
                }
                eprintln!("[exit] nettoyage terminé en {:?}", started.elapsed());
                handle.state::<AppExitCoordinator>().mark_ready();
                handle.exit(code.unwrap_or_default());
            });
        }
    }
}

fn hide_application(app: &tauri::AppHandle) {
    for label in ["main", "mascot"] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.hide();
        }
    }
    #[cfg(target_os = "macos")]
    let _ = app.set_dock_visibility(false);
}

async fn cleanup_services(app: &tauri::AppHandle) {
    cancel_active_streams(app).await;
    if let Some(downloads) = app.try_state::<services::model_downloads::ModelDownloadManager>() {
        downloads.cancel_all().await;
    }
    services::agent_local::tool_bash_profile::clear();
    services::mcp_oauth::flow::cancel_all();

    let gateway = async {
        if let Some(service) = app.try_state::<GatewayService>() {
            service.stop().await;
        }
    };
    let chronos = async {
        if let Some(sidecar) = app.try_state::<services::forecast::sidecar::ChronosSidecar>() {
            services::forecast::sidecar::stop(sidecar.inner()).await;
        }
    };
    let searxng = async {
        if let Some(sidecar) = app.try_state::<services::searxng::SearxngSidecar>() {
            services::searxng::stop(sidecar.inner()).await;
        }
    };
    let terminal_handle = app.clone();
    let terminals = tokio::task::spawn_blocking(move || {
        if let Some(pty) = terminal_handle.try_state::<services::terminal::PtyManager>() {
            pty.kill_all();
        }
    });
    let ollama_handle = app.clone();
    let ollama = tokio::task::spawn_blocking(move || {
        services::ollama_lifecycle::stop_sidecar(&ollama_handle);
    });

    let _ = tokio::join!(
        services::oauth_providers::cancel_all(),
        services::codex_oauth::login::cancel_login(),
        services::agent_local::tool_bash_registry::stop_all(),
        services::mcp_bridge::process_manager::shutdown_all(),
        services::extensions::stop(),
        services::ollama_kill::release_vram(),
        gateway,
        chronos,
        searxng,
        terminals,
        ollama,
    );
}

async fn cancel_active_streams(app: &tauri::AppHandle) {
    let Some(streams) = app.try_state::<ActiveStreams>() else {
        return;
    };
    let active = {
        let mut streams = streams.0.lock().await;
        streams.drain().map(|(_, entry)| entry).collect::<Vec<_>>()
    };
    for (cancel, _, _, _) in active {
        cancel.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_starts_once_and_only_exits_when_ready() {
        let state = AppExitCoordinator::default();
        assert_eq!(state.begin(), BeginResult::Started);
        assert_eq!(state.begin(), BeginResult::Waiting);
        state.mark_ready();
        assert_eq!(state.begin(), BeginResult::Ready);
    }
}
