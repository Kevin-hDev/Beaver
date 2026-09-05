use crate::services::extensions::install_jobs::{
    self, InstallJobView, InstallJobsSnapshot, InstallRequest,
};

#[tauri::command]
pub async fn start_extension_install(request: InstallRequest) -> Result<InstallJobView, String> {
    install_jobs::global()?.start(request)
}
#[tauri::command]
pub fn list_extension_installs() -> Result<InstallJobsSnapshot, String> {
    install_jobs::global()?.snapshot()
}
#[tauri::command]
pub fn cancel_extension_install(job_id: String) -> Result<InstallJobView, String> {
    install_jobs::global()?.request_cancel(&job_id)
}
#[tauri::command]
pub fn continue_extension_install(
    job_id: String,
    confirmation_id: String,
) -> Result<InstallJobView, String> {
    install_jobs::global()?.confirm(&job_id, &confirmation_id)
}
#[tauri::command]
pub fn dismiss_extension_install(job_id: String) -> Result<(), String> {
    install_jobs::global()?.dismiss(&job_id)
}
#[tauri::command]
pub fn resume_extension_install(job_id: String) -> Result<InstallJobView, String> {
    install_jobs::global()?.resume(&job_id)
}
