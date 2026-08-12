use std::collections::HashMap;
use tauri::Manager;
use tokio::sync::Mutex;

pub struct ActiveStreams(
    pub(crate) Mutex<HashMap<String, super::commands::agent_chat_streams::StreamEntry>>,
);

pub fn agent_work(
    exit: &crate::app_exit::AppExitCoordinator,
) -> crate::services::agent_local::agent_work_supervision::AgentWorkServices {
    crate::services::agent_local::agent_work_supervision::AgentWorkServices::new(
        exit.work_supervisor(),
    )
}

// Une seule construction garantit que tous les services partagent le superviseur de fermeture.
pub struct RuntimeServices {
    pub agent_work: crate::services::agent_local::agent_work_supervision::AgentWorkServices,
    pub gateway: crate::services::gateway::GatewayService,
    pub oauth_work: crate::services::oauth_work::OAuthWorkServices,
    pub searxng: crate::services::searxng::SearxngSidecar,
    pub downloads: crate::services::model_downloads::ModelDownloadManager,
    pub forecast: crate::services::forecast::sidecar::ChronosSidecar,
    pub app_update: crate::services::update_handoff::AppUpdateRuntime,
}

pub fn services(exit: &crate::app_exit::AppExitCoordinator) -> RuntimeServices {
    let supervisor = exit.work_supervisor();
    RuntimeServices {
        agent_work: agent_work(exit),
        gateway: crate::services::gateway::GatewayService::new(supervisor.clone()),
        oauth_work: crate::services::oauth_work::OAuthWorkServices::new(supervisor.clone()),
        searxng: crate::services::searxng::SearxngSidecar::new(supervisor.clone()),
        downloads: crate::services::model_downloads::ModelDownloadManager::new(supervisor.clone()),
        forecast: crate::services::forecast::sidecar::ChronosSidecar::new(supervisor.clone()),
        app_update: crate::services::update_handoff::AppUpdateRuntime::new(supervisor),
    }
}

pub fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub fn scheduler(app: &tauri::AppHandle) -> std::io::Result<crate::services::scheduler::Scheduler> {
    let exit = app.state::<crate::app_exit::AppExitCoordinator>();
    crate::services::scheduler::Scheduler::spawn(app.clone(), exit.work_supervisor())
        .map_err(std::io::Error::other)
}

pub fn initialize_agent_runtime(app: &tauri::AppHandle) -> std::io::Result<()> {
    crate::services::agent_local::shell_sandbox::cleanup_stale();
    crate::services::agent_local::app_handle_global::init(app.clone());
    crate::services::agent_local::subagent_spawn_channel::init(app).map_err(std::io::Error::other)
}
