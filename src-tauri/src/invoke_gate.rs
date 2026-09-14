pub const UPDATE_PROGRESS_WINDOW: &str = "update-progress";

const UPDATE_PROGRESS_COMMANDS: [&str; 7] = [
    "list_update_operations",
    "dismiss_update_operation",
    "request_update_operation_retry",
    "resize_update_progress_window",
    "cancel_app_update_download",
    "cancel_ollama_setup",
    "cancel_model_download",
];

pub fn allowed(window_label: &str, command: &str) -> bool {
    window_label != UPDATE_PROGRESS_WINDOW || UPDATE_PROGRESS_COMMANDS.contains(&command)
}

pub fn resize_allowed(window_label: &str, height: u16) -> bool {
    window_label == UPDATE_PROGRESS_WINDOW && (96..=640).contains(&height)
}

pub fn wrap<F>(handler: F) -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync
where
    F: Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool + Send + Sync + 'static,
{
    move |invoke| {
        let accepted = allowed(
            invoke.message.webview_ref().label(),
            invoke.message.command(),
        );
        if !accepted {
            invoke.resolver.reject("command-not-available");
            return true;
        }
        handler(invoke)
    }
}
