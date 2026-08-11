use super::super::cef_app::BrowserApp;
use super::super::cef_child_admission::BrowserCefSupervision;
use super::super::cef_engine_config::{prepare_profile, to_cef_settings};
use super::super::cef_preflight::{run_with_retry, CefPreflightError};
#[cfg(target_os = "windows")]
use super::super::cef_unavailable::CefUnavailableCategory;
use super::super::pump_scheduler::PumpScheduler;
use super::super::runtime_handle::BrowserRuntimeHandle;
use super::super::settings::cef_settings_policy;
#[cfg(target_os = "macos")]
use super::super::BrowserLibraryGuard;
use cef::{args::Args, *};
use std::path::PathBuf;

pub(super) enum CefStartFailure {
    Preflight(CefPreflightError),
    Fatal,
}

struct PreparedCef {
    profile: PathBuf,
    helper: PathBuf,
    #[cfg(target_os = "windows")]
    sandbox_info: *mut u8,
    #[cfg(target_os = "windows")]
    tracker: super::super::cef_supervision::WindowsCefTracker,
    #[cfg(target_os = "macos")]
    tracker: super::super::cef_supervision::MacCefTracker,
}

pub(super) fn start(
    app: tauri::AppHandle,
    runtime: BrowserRuntimeHandle,
    #[cfg(target_os = "macos")] library: &BrowserLibraryGuard,
) -> Result<(), CefStartFailure> {
    if super::ENGINE.with(|engine| engine.borrow().is_some()) {
        return Err(CefStartFailure::Fatal);
    }
    let prepared = run_with_retry(
        || {
            prepare_once(
                app.clone(),
                #[cfg(target_os = "macos")]
                library,
            )
        },
        std::thread::sleep,
    )
    .map_err(CefStartFailure::Preflight)?;
    if !runtime.mark_supervised() {
        return Err(CefStartFailure::Fatal);
    }
    initialize_cef(
        app,
        runtime,
        prepared,
        #[cfg(target_os = "macos")]
        library,
    )
}

fn prepare_once(
    app: tauri::AppHandle,
    #[cfg(target_os = "macos")] library: &BrowserLibraryGuard,
) -> Result<PreparedCef, CefPreflightError> {
    #[cfg(target_os = "windows")]
    let helper = {
        let executable = std::env::current_exe()
            .map_err(|error| CefPreflightError::from_io(CefUnavailableCategory::Object, &error))?;
        super::super::native_paths_windows_preflight::resolve_runtime_files(&executable)?.helper
    };
    #[cfg(target_os = "macos")]
    let helper = library.runtime_files().helper.clone();
    let profile = prepare_profile()?;
    #[cfg(target_os = "windows")]
    let sandbox_info = super::super::windows_sandbox::get()
        .ok_or_else(|| CefPreflightError::deterministic(CefUnavailableCategory::Sandbox))?;

    // The tracker is intentionally last: a retried attempt cannot reuse any
    // native supervision object created by the previous attempt.
    #[cfg(target_os = "windows")]
    let tracker = super::super::cef_supervision::WindowsCefTracker::start_supervised(&helper, app)?;
    #[cfg(target_os = "macos")]
    let tracker = super::super::cef_supervision::MacCefTracker::start_supervised(
        &library.runtime_files().supervised_helpers,
        super::super::cef_runtime_policy::cef_supervision_root(),
        app,
    )?;
    Ok(PreparedCef {
        profile,
        helper,
        #[cfg(target_os = "windows")]
        sandbox_info,
        tracker,
    })
}

fn initialize_cef(
    app: tauri::AppHandle,
    runtime: BrowserRuntimeHandle,
    prepared: PreparedCef,
    #[cfg(target_os = "macos")] library: &BrowserLibraryGuard,
) -> Result<(), CefStartFailure> {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = Args::new();
    #[cfg(target_os = "macos")]
    {
        let result = execute_process(
            Some(args.as_main_args()),
            None::<&mut App>,
            std::ptr::null_mut(),
        );
        super::super::process_role::validate_browser_process_result(result)
            .map_err(|_| CefStartFailure::Fatal)?;
    }

    let pump = PumpScheduler::new(app);
    let supervision = BrowserCefSupervision::new(prepared.tracker.handle());
    let mut cef_app = BrowserApp::new(pump.clone(), runtime, prepared.profile.clone(), supervision);
    let settings = to_cef_settings(cef_settings_policy(&prepared.profile, &prepared.helper));
    #[cfg(target_os = "macos")]
    let sandbox_info = std::ptr::null_mut();
    #[cfg(target_os = "windows")]
    let sandbox_info = prepared.sandbox_info;
    if cef::initialize(
        Some(args.as_main_args()),
        Some(&settings),
        Some(&mut cef_app),
        sandbox_info,
    ) != 1
    {
        #[cfg(target_os = "macos")]
        library.suppress_unload_after_failed_initialize();
        return Err(CefStartFailure::Fatal);
    }

    let engine = super::CefEngine {
        pump: pump.clone(),
        surface: super::BrowserSurfaceManager::new(),
        _app: cef_app,
        _tracker: prepared.tracker,
    };
    super::ENGINE.with(|slot| {
        *slot.borrow_mut() = Some(engine);
    });
    pump.start_fallback().map_err(|_| CefStartFailure::Fatal)
}
