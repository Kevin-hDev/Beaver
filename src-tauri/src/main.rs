#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(all(feature = "e2e", not(debug_assertions)))]
compile_error!("the e2e feature must never be compiled in release mode");

#[cfg(not(target_os = "windows"))]
fn main() {
    if let Some(code) = cl_go_dash_lib::run_shell_sandbox_helper() {
        std::process::exit(code);
    }
    // SAFETY: le helper éventuel ne crée aucun thread et remplace le processus
    // avec exec ; cet appel reste donc antérieur à CEF, Tauri et tout thread.
    if !unsafe { cl_go_dash_lib::configure_git_network_policy() } {
        ::log::error!("[git] network policy unavailable");
        return;
    }
    #[cfg(target_os = "macos")]
    let (browser_library, shell_environment_ready) = cl_go_dash_lib::prepare_macos_application();
    #[cfg(not(target_os = "macos"))]
    let shell_environment_ready = cl_go_dash_lib::initialize_shell_environment();
    if !shell_environment_ready {
        ::log::warn!("[shell] login environment unavailable; using fallback PATH");
    }
    #[cfg(target_os = "macos")]
    if browser_library.is_none() {
        ::log::error!("[browser] native integration unavailable");
    }
    #[cfg(not(target_os = "macos"))]
    if !cl_go_dash_lib::prepare_browser_native_application() {
        ::log::error!("[browser] native integration unavailable");
    }
    #[cfg(target_os = "macos")]
    cl_go_dash_lib::run(browser_library);
    #[cfg(not(target_os = "macos"))]
    cl_go_dash_lib::run();
}

#[cfg(target_os = "windows")]
fn main() {
    if let Some(code) = cl_go_dash_lib::run_shell_sandbox_helper() {
        std::process::exit(code);
    }
    std::process::exit(cl_go_dash_lib::launch_windows_browser_host());
}
