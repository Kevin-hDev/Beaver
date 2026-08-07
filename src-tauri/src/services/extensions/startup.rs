use tauri::Emitter;

pub fn initialize_on_startup(app: &tauri::AppHandle) {
    if super::runtime::init(app).is_err() {
        ::log::error!("[extensions] initialization failed");
        return;
    }

    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = super::runtime::start_and_sync().await;
        let _ = handle.emit("fs:extensions-changed", ());
    });
}
