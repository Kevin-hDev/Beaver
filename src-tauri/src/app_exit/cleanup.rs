use super::policy::ShutdownTimeline;
use crate::services::gateway::GatewayService;
use crate::{services, ActiveStreams};
use futures_util::FutureExt;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::time::Instant;
use tauri::Manager;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CleanupOutcome {
    Completed,
    TimedOut,
    Panicked,
}

pub(super) async fn run(app: &tauri::AppHandle, timeline: ShutdownTimeline) -> CleanupOutcome {
    let deadline = timeline.graceful_deadline();
    run_with_deadline(deadline, cleanup_services(app, deadline)).await
}

pub(super) async fn run_with_deadline<Work>(deadline: Instant, work: Work) -> CleanupOutcome
where
    Work: Future,
{
    let guarded = AssertUnwindSafe(work).catch_unwind();
    match tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), guarded).await {
        Err(_) => CleanupOutcome::TimedOut,
        Ok(Err(_)) => CleanupOutcome::Panicked,
        Ok(Ok(_)) => CleanupOutcome::Completed,
    }
}

pub(super) async fn run_ordered<Services, Ollama>(services: Services, ollama: Ollama)
where
    Services: Future,
    Ollama: Future,
{
    services.await;
    ollama.await;
}

async fn cleanup_services(app: &tauri::AppHandle, deadline: Instant) {
    cancel_active_streams(app).await;
    services::agent_local::tool_bash_profile::clear();
    let services_phase = stop_services(app, deadline);
    let ollama_handle = app.clone();
    let ollama_phase = async move {
        super::blocking::execute(move || {
            services::ollama_lifecycle::stop_sidecar(&ollama_handle);
        })
        .await;
    };
    run_ordered(services_phase, ollama_phase).await;
}

async fn stop_services(app: &tauri::AppHandle, deadline: Instant) {
    let agent_work = async {
        if let Some(work) =
            app.try_state::<services::agent_local::agent_work_supervision::AgentWorkServices>()
        {
            let _ = work.stop_and_wait(deadline).await;
        }
    };
    let gateway = async {
        if let Some(service) = app.try_state::<GatewayService>() {
            let _ = service.stop_and_wait(deadline).await;
        }
    };
    let chronos = async {
        if let Some(sidecar) = app.try_state::<services::forecast::sidecar::ChronosSidecar>() {
            services::forecast::sidecar::stop(sidecar.inner()).await;
        }
    };
    let downloads = async {
        if let Some(manager) = app.try_state::<services::model_downloads::ModelDownloadManager>() {
            let _ = manager.stop_and_wait(deadline).await;
        }
    };
    let searxng = async {
        if let Some(sidecar) = app.try_state::<services::searxng::SearxngSidecar>() {
            let _ = sidecar.stop_and_wait(deadline).await;
        }
    };
    let terminal_handle = app.clone();
    let terminals = super::blocking::execute(move || {
        if let Some(pty) = terminal_handle.try_state::<services::terminal::PtyManager>() {
            pty.kill_all();
        }
    });
    let oauth = async {
        if let Some(work) = app.try_state::<services::oauth_work::OAuthWorkServices>() {
            let _ = work.stop_and_wait(deadline).await;
        }
    };

    let _ = tokio::join!(
        services::agent_local::tool_bash_registry::stop_all(),
        services::mcp_bridge::process_manager::stop_and_wait(deadline),
        services::extensions::stop_and_wait(deadline),
        services::ollama_kill::release_vram(),
        gateway,
        downloads,
        chronos,
        searxng,
        terminals,
        oauth,
        agent_work,
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
