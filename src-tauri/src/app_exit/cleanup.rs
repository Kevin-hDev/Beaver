use super::policy::ShutdownTimeline;
use crate::services;
use crate::services::gateway::GatewayService;
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

pub(super) type StopFuture<'a> = futures_util::future::BoxFuture<'a, bool>;

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
            work.stop_and_wait(deadline).await
        } else {
            true
        }
    };
    let gateway = async {
        if let Some(service) = app.try_state::<GatewayService>() {
            service.stop_and_wait(deadline).await
        } else {
            true
        }
    };
    let chronos = async {
        if let Some(sidecar) = app.try_state::<services::forecast::sidecar::ChronosSidecar>() {
            sidecar.stop_and_wait(deadline).await
        } else {
            true
        }
    };
    let downloads = async {
        if let Some(manager) = app.try_state::<services::model_downloads::ModelDownloadManager>() {
            manager.stop_and_wait(deadline).await
        } else {
            true
        }
    };
    let app_update = async {
        if let Some(runtime) = app.try_state::<services::update_handoff::AppUpdateRuntime>() {
            let stopped = runtime.stop_and_wait(deadline).await;
            if let Some(identity) = runtime.transferred_identity() {
                let mut system = sysinfo::System::new();
                let pid = sysinfo::Pid::from_u32(identity.pid());
                system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
                if identity.is_current(&system) {
                    ::log::info!("[update] helper transféré préservé pid={}", identity.pid());
                }
            }
            stopped
        } else {
            true
        }
    };
    let searxng = async {
        if let Some(sidecar) = app.try_state::<services::searxng::SearxngSidecar>() {
            sidecar.stop_and_wait(deadline).await
        } else {
            true
        }
    };
    let terminals = async {
        if let Some(pty) = app.try_state::<services::terminal::PtyManager>() {
            pty.stop_and_wait(deadline).await
        } else {
            true
        }
    };
    let oauth = async {
        if let Some(work) = app.try_state::<services::oauth_work::OAuthWorkServices>() {
            work.stop_and_wait(deadline).await
        } else {
            true
        }
    };
    let scheduler = async {
        if let Some(scheduler) = app.try_state::<services::scheduler::Scheduler>() {
            scheduler.stop_and_wait(deadline).await
        } else {
            true
        }
    };
    let background = async {
        if let Some(work) =
            app.try_state::<services::runtime_background::RuntimeBackgroundServices>()
        {
            work.stop_and_wait(deadline).await
        } else {
            true
        }
    };

    let service_stops = [
        ("agent-work", agent_work.boxed()),
        ("gateway", gateway.boxed()),
        ("forecast", chronos.boxed()),
        ("model-downloads", downloads.boxed()),
        ("app-update", app_update.boxed()),
        ("searxng", searxng.boxed()),
        ("terminal", terminals.boxed()),
        ("oauth", oauth.boxed()),
        ("scheduler", scheduler.boxed()),
        ("runtime-background", background.boxed()),
        (
            "mcp",
            services::mcp_bridge::process_manager::stop_and_wait(deadline).boxed(),
        ),
        (
            "extensions",
            services::extensions::stop_and_wait(deadline).boxed(),
        ),
    ];
    let (all_stopped, ()) = tokio::join!(
        run_service_group(service_stops),
        services::ollama_kill::release_vram(),
    );
    if !all_stopped {
        ::log::warn!("[exit] one or more services exceeded the graceful deadline");
    }
    verify_global_registry(app);
}

pub(super) async fn run_service_group<'a, const N: usize>(
    services: [(&'static str, StopFuture<'a>); N],
) -> bool {
    let results =
        futures_util::future::join_all(services.into_iter().map(|(name, stop)| async move {
            let stopped = stop.await;
            if !stopped {
                ::log::warn!("[exit] service {name} exceeded the graceful deadline");
            }
            stopped
        }))
        .await;
    results.into_iter().all(std::convert::identity)
}

fn verify_global_registry(app: &tauri::AppHandle) {
    let Some(coordinator) = app.try_state::<super::AppExitCoordinator>() else {
        ::log::warn!("[exit] global work registry unavailable after service cleanup");
        return;
    };
    let _ = global_registry_is_empty(coordinator.registry.active_count());
}

pub(super) fn global_registry_is_empty(active_count: usize) -> bool {
    if active_count == 0 {
        ::log::info!("[exit] global work registry empty");
        true
    } else {
        ::log::warn!("[exit] global work registry still active count={active_count}");
        false
    }
}
