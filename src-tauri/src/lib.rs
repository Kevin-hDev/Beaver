#![cfg_attr(all(test, windows), windows_subsystem = "windows")]
// Plusieurs modules de tests compagnons portent le même nom que leur module
// parent (convention *_tests.rs). C'est intentionnel et documenté.
#![allow(clippy::module_inception)]

mod app_build;
mod app_events;
mod app_exit;
mod app_lifecycle;
mod commands;
mod invoke_handler;
mod invoke_handler_tail;
#[cfg(target_os = "macos")]
mod macos_app_menu;
#[cfg(target_os = "macos")]
mod macos_termination;
mod models;
mod runtime_startup;
mod runtime_state;
mod services;
mod startup;
#[cfg(test)]
mod startup_tests;
mod storage_default_skills;
mod storage_migration;
mod storage_migration_files;
mod tray;
pub mod updater_worker;
#[cfg(all(target_os = "windows", not(feature = "windows-tests")))]
mod windows_entry;
#[cfg(all(test, target_os = "windows", feature = "windows-tests"))]
#[path = "windows_entry_plan.rs"]
mod windows_entry_plan;

pub use runtime_state::ActiveStreams;
#[cfg(target_os = "macos")]
pub use services::browser::BrowserLibraryGuard;
#[cfg(target_os = "macos")]
pub fn run_macos_cef_helper() -> std::process::ExitCode {
    services::browser::run_macos_cef_helper()
}
#[cfg(all(target_os = "windows", not(feature = "windows-tests")))]
pub use startup::launch_windows_browser_host;
#[cfg(target_os = "macos")]
pub use startup::prepare_macos_application;
#[cfg(not(target_os = "windows"))]
pub use startup::run;
pub use startup::{
    configure_git_network_policy, initialize_shell_environment, prepare_browser_native_application,
    run_shell_sandbox_helper, run_terminal_shell_helper_if_requested,
};

#[cfg(debug_assertions)]
pub fn run_live_reasoning_fixtures() -> bool {
    let coordinator = match app_exit::AppExitCoordinator::initialize() {
        Ok(value) => value,
        Err(_) => return false,
    };
    let runtime = runtime_state::services(&coordinator);
    let app = match app_build::build_live_fixture(coordinator, runtime) {
        Ok(value) => value,
        Err(_) => return false,
    };
    services::agent_local::app_handle_global::init(app.handle().clone());
    match tauri::async_runtime::block_on(
        commands::reasoning_fixture_live_tests::refresh_live_reasoning_fixture_matrix_once(&app),
    ) {
        Ok(()) => true,
        Err(error) => {
            eprintln!("{error}");
            false
        }
    }
}

pub(crate) fn run_inner(
    #[cfg(target_os = "macos")] browser_library: Option<BrowserLibraryGuard>,
    ui_startup: services::extensions::UiStartupState,
) -> bool {
    let exit_coordinator = match app_exit::AppExitCoordinator::initialize() {
        Ok(coordinator) => coordinator,
        Err(_) => {
            eprintln!("[exit] safety initialization unavailable");
            return false;
        }
    };
    if services::mcp_bridge::process_manager::init(exit_coordinator.work_supervisor()).is_err() {
        eprintln!("[mcp] shutdown supervision unavailable");
        return false;
    }
    let runtime = runtime_state::services(&exit_coordinator);
    std::hint::black_box(tauri::utils::platform::bundle_type());
    let app = match app_build::build(exit_coordinator, runtime, ui_startup) {
        Ok(app) => app,
        Err(_) => {
            eprintln!("[app] initialization failed");
            return false;
        }
    };

    let exit_code = app_lifecycle::run(
        app,
        #[cfg(target_os = "macos")]
        browser_library,
    );
    std::process::exit(exit_code);
}
