use super::BEAVER_RESTART_REQUEST_CODE;
use tauri::Manager;

pub fn request(app: &tauri::AppHandle, code: i32) {
    let (intent, exit_code) = super::request_flow::requested_intent(Some(code));
    let Some(coordinator) = app.try_state::<super::AppExitCoordinator>() else {
        super::raw_exit::terminate_process(1);
    };
    // Arm the independent raw-exit thread before asking the event loop to unwind.
    if !coordinator.prearm_request(intent, exit_code) {
        super::raw_exit::terminate_process(1);
    }
    app.exit(code);
}

pub fn request_restart(app: &tauri::AppHandle) {
    request(app, BEAVER_RESTART_REQUEST_CODE);
}
