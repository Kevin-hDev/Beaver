use super::BEAVER_RESTART_REQUEST_CODE;
use tauri::Manager;

pub fn request(app: &tauri::AppHandle, code: i32) {
    if try_request(app, code).is_err() {
        super::raw_exit::terminate_process(1);
    }
}

pub(crate) fn try_request(app: &tauri::AppHandle, code: i32) -> Result<(), ()> {
    let (intent, exit_code) = super::request_flow::requested_intent(Some(code));
    let coordinator = app.try_state::<super::AppExitCoordinator>().ok_or(())?;
    // Arm the independent raw-exit thread before asking the event loop to unwind.
    if !coordinator.prearm_request(intent, exit_code) {
        return Err(());
    }
    app.exit(code);
    Ok(())
}

pub fn request_restart(app: &tauri::AppHandle) {
    request(app, BEAVER_RESTART_REQUEST_CODE);
}
