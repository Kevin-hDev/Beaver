#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(cl_go_dash_lib::updater_worker::run_from_env);
    std::panic::set_hook(previous_hook);
    if !matches!(result, Ok(Ok(()))) {
        eprintln!("update failed");
        std::process::exit(1);
    }
}
