#[cfg(any(target_os = "macos", target_os = "windows"))]
use super::BrowserRuntimeHandle;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use tauri::Manager;

#[cfg(any(target_os = "macos", target_os = "windows"))]
static NATIVE_APPLICATION_READY: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "macos")]
pub(crate) fn prepare_native_application() -> bool {
    let ready = super::native_application::prepare().is_ok();
    NATIVE_APPLICATION_READY.store(ready, Ordering::Release);
    ready
}

#[cfg(target_os = "windows")]
pub(crate) fn prepare_native_application() -> bool {
    NATIVE_APPLICATION_READY.store(true, Ordering::Release);
    true
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) fn prepare_native_application() -> bool {
    true
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) fn setup_on_run_event(app: &tauri::AppHandle, event: &tauri::RunEvent) {
    if !matches!(event, tauri::RunEvent::Ready) {
        return;
    }
    let runtime = app.state::<BrowserRuntimeHandle>().inner().clone();
    if !NATIVE_APPLICATION_READY.load(Ordering::Acquire) || !runtime.mark_application_prepared() {
        return;
    }
    super::cef_engine::initialize(app.clone(), runtime);
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) fn setup_on_run_event(_app: &tauri::AppHandle, _event: &tauri::RunEvent) {}

pub(crate) fn reset_page_surface(_app: &tauri::AppHandle) {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        let app = _app;
        let main_app = app.clone();
        if app
            .run_on_main_thread(move || {
                if super::cef_engine::reset_page_surface(&main_app).is_err() {
                    eprintln!("[browser] surface reset failed");
                }
            })
            .is_err()
        {
            eprintln!("[browser] surface reset unavailable");
        }
    }
}

pub(crate) fn shutdown(_app: &tauri::AppHandle) {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    super::cef_engine::shutdown(_app.state::<BrowserRuntimeHandle>().inner());
}
