use std::collections::HashMap;
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

pub fn initialize_agent_runtime(app: &tauri::AppHandle) -> std::io::Result<()> {
    crate::services::agent_local::shell_sandbox::cleanup_stale();
    crate::services::agent_local::app_handle_global::init(app.clone());
    crate::services::agent_local::subagent_spawn_channel::init(app).map_err(std::io::Error::other)
}
