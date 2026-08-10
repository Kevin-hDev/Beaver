use tauri::Manager;

pub(super) fn hide_application(app: &tauri::AppHandle) {
    for label in ["main", "mascot"] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.hide();
        }
    }
    #[cfg(target_os = "macos")]
    let _ = app.set_dock_visibility(false);
}
