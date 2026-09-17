mod favicon_events;
pub use favicon_events::{read_snapshot as favicon_snapshot, BrowserFaviconSnapshot};
mod browser_api_types;
#[cfg(any(test, native_browser))]
mod browser_events;
#[cfg(native_browser)]
mod browser_slot;
mod browser_surface_api;
#[cfg(any(test, native_browser))]
mod browser_view_key;
#[cfg(native_browser)]
mod cef_app;
#[cfg(native_browser)]
mod cef_blocked_feature;
#[cfg(native_browser)]
mod cef_child_admission;
#[cfg(native_browser)]
mod cef_client;
#[cfg(native_browser)]
mod cef_cookie_gate;
#[cfg(native_browser)]
mod cef_cookie_gate_cleanup;
#[cfg(any(test, browser_native_api))]
mod cef_cookie_gate_policy;
#[cfg(native_browser)]
mod cef_diagnostics;
#[cfg(native_browser)]
mod cef_display_handler;
#[cfg(native_browser)]
mod cef_download_handler;
#[cfg(native_browser)]
mod cef_engine;
#[cfg(native_browser)]
mod cef_engine_config;
#[cfg(native_browser)]
mod cef_favicon_candidates;
#[cfg(native_browser)]
mod cef_favicon_handler;
#[cfg(native_browser)]
mod cef_favicon_image;
#[cfg(native_browser)]
mod cef_favicon_scheduler;
#[cfg(target_os = "macos")]
mod cef_library;
#[cfg(native_browser)]
mod cef_life_span_handler;
#[cfg(native_browser)]
mod cef_load_handler;
#[cfg(native_browser)]
mod cef_permission_handler;
#[cfg(any(test, browser_native_api))]
mod cef_preflight;
#[cfg(native_browser)]
mod cef_request_handler;
mod cef_runtime_policy;
#[cfg(native_browser)]
mod cef_state_bridge;
#[cfg(any(test, native_browser))]
mod favicon_png;
#[cfg(any(test, native_browser))]
mod favicon_policy;
#[cfg(test)]
mod favicon_review_tests;
#[cfg(any(test, native_browser))]
mod favicon_runtime;
#[cfg(any(test, native_browser))]
mod favicon_state;
#[cfg(test)]
mod favicon_state_tests;
#[cfg(any(test, native_browser))]
mod favicon_store;
#[cfg(any(test, native_browser))]
mod favicon_task_gate;
#[cfg(any(test, native_browser))]
mod favicon_types;
#[cfg(any(test, native_browser))]
mod favicon_watchdog;
// La supervision reste disponible avec l'API native sous windows-tests ;
// native_browser désigne uniquement le moteur CEF réellement lié.
#[cfg(any(test, browser_native_api))]
mod cef_supervision;
#[cfg(native_browser)]
mod cef_surface;
#[cfg(native_browser)]
mod cef_surface_view;
#[cfg(native_browser)]
mod cef_text;
#[cfg(any(test, browser_native_api))]
#[path = "cef_supervision/diagnostics.rs"]
mod cef_unavailable;
#[cfg(any(test, target_os = "macos"))]
mod cookie_store_probe;
#[cfg(browser_native_api)]
mod ffi_guard;
#[cfg(any(test, browser_native_api))]
mod lifecycle;
mod live_session_registry;
mod local_site_candidates;
mod local_site_policy;
mod local_site_probe;
mod local_site_scan_state;
mod local_site_scan_throttle;
mod local_site_scanner;
mod local_site_types;
#[cfg(target_os = "macos")]
mod macos_helper_entry;
#[cfg(target_os = "macos")]
mod native_application;
#[cfg(any(test, browser_native_api))]
mod native_paths;
#[cfg(target_os = "macos")]
mod native_paths_macos_preflight;
#[cfg(all(native_browser, target_os = "windows"))]
mod native_paths_windows_preflight;
#[cfg(target_os = "macos")]
mod native_pump;
#[cfg(target_os = "macos")]
mod native_pump_wake;
#[cfg(native_browser)]
mod native_surface;
#[cfg(any(test, browser_native_api))]
mod navigation_target;
pub(crate) mod process_role;
pub(crate) use process_role::observe_native_webviews;
#[cfg(target_os = "macos")]
mod pump_gate;
#[cfg(native_browser)]
mod pump_scheduler;
mod runtime_handle;
mod runtime_integration;
#[cfg(any(test, browser_native_api))]
mod runtime_revision;
mod session_model;
mod session_model_persistence;
#[cfg(any(test, browser_native_api))]
mod session_model_runtime;
mod session_order;
mod session_persistence;
mod session_service;
#[cfg(browser_native_api)]
mod session_service_runtime;
mod session_store;
mod session_types;
mod session_validation;
#[cfg(browser_native_api)]
mod settings;
mod surface_bounds;
mod tab_id;
mod url_policy;
#[cfg(any(test, browser_native_api))]
mod view_recency;
#[cfg(any(test, browser_native_api))]
mod view_state;
#[cfg(target_os = "windows")]
pub(crate) mod windows_sandbox;
#[cfg(target_os = "windows")]
mod windows_surface_order;

// Keep test registration separate from the production module boundary.
#[cfg(test)]
mod test_modules;

use tauri::Manager;

pub use browser_api_types::{BrowserCommandError, BrowserNavigationAction, BrowserSurfaceRequest};
pub use browser_surface_api::{
    apply_surface, close_native_view, navigate_native_view, run_navigation_action,
};
#[cfg(target_os = "macos")]
pub use cef_library::BrowserLibraryGuard;
pub(crate) use cef_runtime_policy::{
    begin_cef_shutdown, cef_has_runnable_helpers, force_cef_shutdown, CefShutdownBarrier,
};
#[cfg(all(native_browser, target_os = "windows"))]
pub(crate) use cef_supervision::{WindowsHelperAdmission, CEF_ADMISSION_SWITCH};
pub use local_site_scanner::LocalSiteScanner;
pub use local_site_types::{LocalSiteScanResult, LOCAL_SITES_CHANGED_EVENT};
pub use runtime_handle::{BrowserCapability, BrowserRuntimeHandle};
pub(crate) use runtime_integration::{
    prepare_native_application, reset_page_surface, setup_on_run_event, shutdown,
};
pub use session_model::{BrowserSessionState, BrowserTabCreation};
pub use session_service::BrowserSessionService;

#[cfg(feature = "e2e")]
pub(crate) fn seed_e2e_session_key_fixture() -> Result<(), ()> {
    session_store::seed_e2e_session_key_fixture()
}

#[cfg(target_os = "macos")]
pub(crate) use macos_helper_entry::run as run_macos_cef_helper;

pub fn capability(app: &tauri::AppHandle) -> BrowserCapability {
    cef_runtime_policy::capability_for_runtime(app.state::<BrowserRuntimeHandle>().inner())
}
