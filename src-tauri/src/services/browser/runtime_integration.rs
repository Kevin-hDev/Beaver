#[cfg(target_os = "macos")]
use super::BrowserLibraryGuard;
#[cfg(native_browser)]
use super::BrowserRuntimeHandle;
#[cfg(native_browser)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(native_browser)]
use tauri::Manager;

#[cfg(native_browser)]
static NATIVE_APPLICATION_READY: AtomicBool = AtomicBool::new(false);

#[cfg(any(test, native_browser))]
pub(super) fn is_browser_ready_event(event: &tauri::RunEvent) -> bool {
    matches!(event, tauri::RunEvent::Ready)
}

#[cfg(target_os = "macos")]
pub(crate) fn prepare_native_application() -> bool {
    let ready = super::native_application::prepare().is_ok();
    NATIVE_APPLICATION_READY.store(ready, Ordering::Release);
    ready
}

#[cfg(all(target_os = "windows", not(feature = "windows-tests")))]
pub(crate) fn prepare_native_application() -> bool {
    NATIVE_APPLICATION_READY.store(true, Ordering::Release);
    true
}

#[cfg(not(native_browser))]
pub(crate) fn prepare_native_application() -> bool {
    true
}

#[cfg(native_browser)]
pub(crate) fn setup_on_run_event(
    app: &tauri::AppHandle,
    event: &tauri::RunEvent,
    #[cfg(target_os = "macos")] library: Option<&BrowserLibraryGuard>,
) {
    if !is_browser_ready_event(event) {
        return;
    }
    #[cfg(target_os = "macos")]
    let Some(library) = library
    else {
        return;
    };
    let runtime = app.state::<BrowserRuntimeHandle>().inner().clone();
    if !NATIVE_APPLICATION_READY.load(Ordering::Acquire) || !runtime.mark_application_prepared() {
        return;
    }
    super::cef_engine::initialize(
        app.clone(),
        runtime,
        #[cfg(target_os = "macos")]
        library,
    );
}

#[cfg(not(native_browser))]
pub(crate) fn setup_on_run_event(
    _app: &tauri::AppHandle,
    _event: &tauri::RunEvent,
    #[cfg(target_os = "macos")] _library: Option<&BrowserLibraryGuard>,
) {
}

pub(crate) fn reset_page_surface(_app: &tauri::AppHandle) {
    #[cfg(native_browser)]
    {
        let app = _app;
        let main_app = app.clone();
        if app
            .run_on_main_thread(move || {
                if super::cef_engine::reset_page_surface(&main_app).is_err() {
                    ::log::error!("[browser] surface reset failed");
                }
            })
            .is_err()
        {
            ::log::error!("[browser] surface reset unavailable");
        }
    }
}

pub(crate) fn shutdown(_app: &tauri::AppHandle) {
    #[cfg(native_browser)]
    super::cef_engine::shutdown(_app.state::<BrowserRuntimeHandle>().inner());
}
