use super::cef_app::BrowserApp;
use super::cef_engine_config::{prepare_profile, to_cef_settings};
#[cfg(target_os = "macos")]
use super::cef_library::CefLibrary;
use super::cef_surface::BrowserSurfaceManager;
use super::native_paths::resolve_runtime_files;
#[cfg(target_os = "macos")]
use super::process_role::validate_browser_process_result;
use super::pump_scheduler::PumpScheduler;
use super::runtime_handle::BrowserRuntimeHandle;
use super::settings::cef_settings_policy;
use super::surface_bounds::BrowserSurfaceBounds;
use super::url_policy::ValidatedUrl;
use super::{browser_api_types::BrowserNavigationAction, browser_view_key::BrowserViewKey};
use cef::{args::Args, *};
use std::cell::RefCell;

thread_local! {
    static ENGINE: RefCell<Option<CefEngine>> = const { RefCell::new(None) };
}

struct CefEngine {
    pump: PumpScheduler,
    surface: BrowserSurfaceManager,
    _app: App,
    #[cfg(target_os = "macos")]
    _library: CefLibrary,
}

pub(super) fn initialize(app: tauri::AppHandle, runtime: BrowserRuntimeHandle) {
    if initialize_inner(app, runtime.clone()).is_err() {
        let _ = runtime.mark_failed();
        eprintln!("[browser] initialization failed");
    }
}

fn initialize_inner(app: tauri::AppHandle, runtime: BrowserRuntimeHandle) -> Result<(), ()> {
    if ENGINE.with(|engine| engine.borrow().is_some()) {
        return Err(());
    }
    let executable = std::env::current_exe().map_err(|_| ())?;
    let downloaded = cef::sys::get_cef_dir();
    let files = resolve_runtime_files(&executable, downloaded.as_deref()).ok_or(())?;
    let profile = prepare_profile()?;
    #[cfg(target_os = "macos")]
    let library = CefLibrary::load(&files.framework)?;
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = Args::new();
    #[cfg(target_os = "macos")]
    {
        let process_result = execute_process(
            Some(args.as_main_args()),
            None::<&mut App>,
            std::ptr::null_mut(),
        );
        validate_browser_process_result(process_result)?;
    }
    let pump = PumpScheduler::new(app);
    let mut cef_app = BrowserApp::new(pump.clone(), runtime, profile.clone());
    let settings = to_cef_settings(cef_settings_policy(&profile, &files.helper));
    #[cfg(target_os = "macos")]
    let sandbox_info = std::ptr::null_mut();
    #[cfg(target_os = "windows")]
    let sandbox_info = super::windows_sandbox::get().ok_or(())?;
    if cef::initialize(
        Some(args.as_main_args()),
        Some(&settings),
        Some(&mut cef_app),
        sandbox_info,
    ) != 1
    {
        return Err(());
    }
    pump.start_fallback()?;
    ENGINE.with(|engine| {
        *engine.borrow_mut() = Some(CefEngine {
            pump: pump.clone(),
            surface: BrowserSurfaceManager::new(),
            _app: cef_app,
            #[cfg(target_os = "macos")]
            _library: library,
        });
    });
    Ok(())
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
