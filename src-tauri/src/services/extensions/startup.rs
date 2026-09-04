use tauri::Manager;

pub fn initialize_on_startup(app: &tauri::AppHandle) {
    if initialize(app).is_err() {
        ::log::warn!("[extensions] startup refused");
    }
}

pub(crate) fn initialize(app: &tauri::AppHandle) -> Result<(), String> {
    let Some(coordinator) = app.try_state::<crate::app_exit::AppExitCoordinator>() else {
        ::log::error!("[extensions] shutdown supervision unavailable");
        return Err(super::error_codes::HOST_UNAVAILABLE.to_string());
    };
    if let Err(error) = super::runtime::init(app, coordinator.work_supervisor()) {
        ::log::error!("[extensions] initialization failed");
        return Err(error);
    }
    super::runtime_lifecycle::start_background(app.clone())
}
