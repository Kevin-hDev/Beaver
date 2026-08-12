use tauri::Manager;

pub fn initialize_on_startup(app: &tauri::AppHandle) {
    let Some(coordinator) = app.try_state::<crate::app_exit::AppExitCoordinator>() else {
        ::log::error!("[extensions] shutdown supervision unavailable");
        return;
    };
    if super::runtime::init(app, coordinator.work_supervisor()).is_err() {
        ::log::error!("[extensions] initialization failed");
        return;
    }

    if super::runtime_lifecycle::start_background(app.clone()).is_err() {
        ::log::warn!("[extensions] startup refused");
    }
}
