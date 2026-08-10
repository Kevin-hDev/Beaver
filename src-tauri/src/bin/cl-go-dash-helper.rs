#[cfg(target_os = "macos")]
fn main() -> std::process::ExitCode {
    cl_go_dash_lib::run_macos_cef_helper()
}

#[cfg(not(target_os = "macos"))]
fn main() -> std::process::ExitCode {
    std::process::ExitCode::FAILURE
}
