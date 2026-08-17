use super::BEAVER_RESTART_REQUEST_CODE;

pub fn request(app: &tauri::AppHandle, code: i32) {
    app.exit(code);
}

pub fn request_restart(app: &tauri::AppHandle) {
    request_restart_with(|code| app.exit(code));
}

pub(crate) fn request_restart_with(exit: impl FnOnce(i32)) {
    exit(BEAVER_RESTART_REQUEST_CODE);
}
