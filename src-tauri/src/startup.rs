pub fn prepare_browser_native_application() -> bool {
    super::services::browser::prepare_native_application()
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

#[cfg(target_os = "windows")]
pub fn launch_windows_browser_host() -> i32 {
    super::windows_entry::launch_development_bootstrap()
}
