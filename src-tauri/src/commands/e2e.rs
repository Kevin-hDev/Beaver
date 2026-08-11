use std::time::Duration;

const COORDINATED_EXIT_DELAY: Duration = Duration::from_secs(1);

#[tauri::command]
pub fn e2e_request_exit(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(COORDINATED_EXIT_DELAY).await;
        crate::app_exit::request(&app, 0);
    });
}
