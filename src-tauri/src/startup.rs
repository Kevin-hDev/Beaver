#[cfg(target_os = "macos")]
use super::services::browser::BrowserLibraryGuard;

const VAULT_INIT_FAILED_EVENT: &str = "vault-init-failed";

pub(crate) fn emit_vault_init_failed<Emit, Error>(emit: Emit)
where
    Emit: FnOnce(&'static str, ()) -> Result<(), Error>,
{
    let _ = emit(VAULT_INIT_FAILED_EVENT, ());
}

pub fn prepare_browser_native_application() -> bool {
    super::services::browser::prepare_native_application()
}

#[cfg(any(test, target_os = "macos"))]
pub(crate) fn prepare_macos_browser<Guard>(
    load_library: impl FnOnce() -> Result<Guard, ()>,
    prepare_native: impl FnOnce() -> bool,
) -> Option<Guard> {
    let library = load_library().ok()?;
    prepare_native().then_some(library)
}

#[cfg(any(test, target_os = "macos"))]
pub(crate) fn prepare_macos_startup<Guard>(
    load_library: impl FnOnce() -> Result<Guard, ()>,
    prepare_native: impl FnOnce() -> bool,
    initialize_shell: impl FnOnce() -> bool,
) -> (Option<Guard>, bool) {
    let library = prepare_macos_browser(load_library, prepare_native);
    let shell_environment_ready = initialize_shell();
    (library, shell_environment_ready)
}

#[cfg(any(test, target_os = "macos"))]
pub(crate) fn shutdown_before_library_unload<Guard>(
    library: Option<Guard>,
    shutdown: impl FnOnce(),
) {
    shutdown();
    drop(library);
}

pub(crate) fn run_before_browser_shutdown<ExitCode>(
    run_event_loop: impl FnOnce() -> ExitCode,
    shutdown_browser: impl FnOnce(),
    post_browser_cleanup: impl FnOnce(),
) -> ExitCode {
    let exit_code = run_event_loop();
    shutdown_browser();
    post_browser_cleanup();
    exit_code
}

#[cfg(target_os = "macos")]
pub fn prepare_macos_application() -> (Option<BrowserLibraryGuard>, bool) {
    prepare_macos_startup(
        BrowserLibraryGuard::load_for_current_process_with_retry,
        prepare_browser_native_application,
        initialize_shell_environment,
    )
}

#[cfg(target_os = "macos")]
pub fn run(browser_library: Option<BrowserLibraryGuard>) -> bool {
    super::run_inner(browser_library)
}

#[cfg(not(target_os = "macos"))]
pub fn run() -> bool {
    super::run_inner()
}

/// Exécute le lanceur shell isolé avant l'initialisation de Tauri ou de CEF.
pub fn run_shell_sandbox_helper() -> Option<i32> {
    super::services::agent_local::shell_sandbox::run_helper_if_requested()
}

pub fn initialize_shell_environment() -> bool {
    super::services::agent_local::shell_environment::initialize()
}

/// Configure les délais globaux libgit2 avant le démarrage de l'application.
///
/// # Safety
///
/// Doit être appelée avant toute création de thread.
pub unsafe fn configure_git_network_policy() -> bool {
    unsafe { super::services::git::network_policy::configure_before_threads().is_ok() }
}

#[cfg(all(target_os = "windows", not(feature = "windows-tests")))]
pub fn launch_windows_browser_host() -> i32 {
    super::windows_entry::launch_development_bootstrap()
}
