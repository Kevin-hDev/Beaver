use super::cef_surface::BrowserSurfaceManager;
use super::pump_scheduler::PumpScheduler;
use super::runtime_handle::BrowserRuntimeHandle;
use super::surface_bounds::BrowserSurfaceBounds;
use super::url_policy::ValidatedUrl;
#[cfg(target_os = "macos")]
use super::BrowserLibraryGuard;
use super::{browser_api_types::BrowserNavigationAction, browser_view_key::BrowserViewKey};
use cef::App;
use std::cell::RefCell;

mod startup;

thread_local! {
    static ENGINE: RefCell<Option<CefEngine>> = const { RefCell::new(None) };
}

struct CefEngine {
    pump: PumpScheduler,
    surface: BrowserSurfaceManager,
    _app: App,
    #[cfg(target_os = "windows")]
    _tracker: super::cef_supervision::WindowsCefTracker,
    #[cfg(target_os = "macos")]
    _tracker: super::cef_supervision::MacCefTracker,
}

pub(super) fn initialize(
    app: tauri::AppHandle,
    runtime: BrowserRuntimeHandle,
    #[cfg(target_os = "macos")] library: &BrowserLibraryGuard,
) {
    match startup::start(
        app.clone(),
        runtime.clone(),
        #[cfg(target_os = "macos")]
        library,
    ) {
        Ok(()) => {}
        Err(startup::CefStartFailure::Preflight(error)) => {
            let _ = runtime.mark_failed();
            ::log::warn!(
                "[browser] preflight unavailable ({})",
                error.category().code()
            );
            super::cef_runtime_policy::emit_capability(&app, &runtime);
        }
        Err(startup::CefStartFailure::Fatal) => {
            let _ = runtime.mark_failed();
            ::log::error!("[browser] initialization failed after CEF boundary");
            crate::app_exit::request(&app, 1);
        }
    }
}

pub(super) fn apply_surface(
    app: &tauri::AppHandle,
    key: BrowserViewKey,
    url: Option<ValidatedUrl>,
    bounds: BrowserSurfaceBounds,
) -> Result<(), ()> {
    ENGINE.with(|engine| {
        engine
            .borrow_mut()
            .as_mut()
            .ok_or(())?
            .surface
            .apply(app, key, url, bounds)
    })
}

pub(super) fn navigation_action(
    key: &BrowserViewKey,
    action: BrowserNavigationAction,
) -> Result<(), ()> {
    ENGINE.with(|engine| {
        engine
            .borrow_mut()
            .as_mut()
            .ok_or(())?
            .surface
            .action(key, action)
    })
}

pub(super) fn navigate(key: &BrowserViewKey, url: &ValidatedUrl) -> Result<(), ()> {
    ENGINE.with(|engine| {
        engine
            .borrow_mut()
            .as_mut()
            .ok_or(())?
            .surface
            .navigate(key, url)
    })
}

pub(super) fn close_view(app: &tauri::AppHandle, key: &BrowserViewKey) -> Result<(), ()> {
    ENGINE.with(|engine| {
        engine
            .borrow_mut()
            .as_mut()
            .ok_or(())?
            .surface
            .close_view(app, key);
        Ok(())
    })
}

pub(super) fn reset_page_surface(app: &tauri::AppHandle) -> Result<(), ()> {
    ENGINE.with(|engine| {
        let mut engine = engine.borrow_mut();
        let Some(engine) = engine.as_mut() else {
            return Ok(());
        };
        engine.surface.reset_page_surface(app)
    })
}

pub(super) fn shutdown(runtime: &BrowserRuntimeHandle) {
    if !runtime.begin_stopping() {
        return;
    }
    ENGINE.with(|engine| {
        if let Some(engine) = engine.borrow_mut().take() {
            engine.pump.stop();
            let mut engine = engine;
            engine.surface.close();
            #[cfg(target_os = "macos")]
            cef::do_message_loop_work();
            cef::shutdown();
            drop(engine);
        }
    });
    let _ = runtime.mark_stopped();
}
