#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_os = "windows"))]
fn main() {
    if !cl_go_dash_lib::prepare_browser_native_application() {
        eprintln!("[browser] native integration unavailable");
    }
    cl_go_dash_lib::run();
}

#[cfg(target_os = "windows")]
fn main() {
    std::process::exit(cl_go_dash_lib::launch_windows_browser_host());
}
