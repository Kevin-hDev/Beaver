#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(all(feature = "e2e", not(debug_assertions)))]
compile_error!("the e2e feature must never be compiled in release mode");

#[cfg(not(target_os = "windows"))]
fn main() {
    #[cfg(feature = "e2e")]
    eprintln!("[e2e-lifecycle] main-entered");
    // Le plugin de logs Tauri n'existe pas encore ici : garder ces diagnostics sur stderr.
    if let Some(code) = cl_go_dash_lib::run_shell_sandbox_helper() {
        std::process::exit(code);
    }
    if let Some(code) = cl_go_dash_lib::run_terminal_shell_helper_if_requested() {
        std::process::exit(code);
    }
    // SAFETY: le helper éventuel ne crée aucun thread et remplace le processus
    // avec exec ; cet appel reste donc antérieur à CEF, Tauri et tout thread.
    if !unsafe { cl_go_dash_lib::configure_git_network_policy() } {
        eprintln!("[git] network policy unavailable");
        return;
    }
    #[cfg(target_os = "macos")]
    let (browser_library, shell_environment_ready) = cl_go_dash_lib::prepare_macos_application();
    #[cfg(all(feature = "e2e", target_os = "macos"))]
    eprintln!("[e2e-lifecycle] native-prepared");
    #[cfg(not(target_os = "macos"))]
    let shell_environment_ready = cl_go_dash_lib::initialize_shell_environment();
    if !shell_environment_ready {
        eprintln!("[shell] login environment unavailable; using fallback PATH");
    }
    #[cfg(target_os = "macos")]
    if browser_library.is_none() {
        eprintln!("[browser] native integration unavailable");
    }
    #[cfg(not(target_os = "macos"))]
    if !cl_go_dash_lib::prepare_browser_native_application() {
        eprintln!("[browser] native integration unavailable");
    }
    #[cfg(target_os = "macos")]
    let started = cl_go_dash_lib::run(browser_library);
    #[cfg(not(target_os = "macos"))]
    let started = cl_go_dash_lib::run();
    if !started {
        std::process::exit(1);
    }
}

#[cfg(target_os = "windows")]
fn main() {
    if let Some(code) = cl_go_dash_lib::run_shell_sandbox_helper() {
        std::process::exit(code);
    }
    if let Some(code) = cl_go_dash_lib::run_terminal_shell_helper_if_requested() {
        std::process::exit(code);
    }
    std::process::exit(cl_go_dash_lib::launch_windows_browser_host());
}
