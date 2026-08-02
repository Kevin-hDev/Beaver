#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_os = "windows"))]
fn main() {
    if let Some(code) = cl_go_dash_lib::run_shell_sandbox_helper() {
        std::process::exit(code);
    }
    // SAFETY: le helper éventuel ne crée aucun thread et remplace le processus
    // avec exec ; cet appel reste donc antérieur à CEF, Tauri et tout thread.
    if !unsafe { cl_go_dash_lib::configure_git_network_policy() } {
        eprintln!("[git] network policy unavailable");
        return;
    }
    if !cl_go_dash_lib::prepare_browser_native_application() {
        eprintln!("[browser] native integration unavailable");
    }
    cl_go_dash_lib::run();
}

#[cfg(target_os = "windows")]
fn main() {
    if let Some(code) = cl_go_dash_lib::run_shell_sandbox_helper() {
        std::process::exit(code);
    }
    std::process::exit(cl_go_dash_lib::launch_windows_browser_host());
}
