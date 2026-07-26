#[cfg(target_os = "macos")]
fn main() -> std::process::ExitCode {
    use cef::{args::Args, *};

    let args = Args::new();
    let mut sandbox = cef::sandbox::Sandbox::new();
    sandbox.initialize(args.as_main_args());
    let loader = cef::library_loader::LibraryLoader::new(
        &std::env::current_exe().expect("helper executable unavailable"),
        true,
    );
    if !loader.load() {
        return std::process::ExitCode::FAILURE;
    }
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let code = execute_process(
        Some(args.as_main_args()),
        None::<&mut App>,
        std::ptr::null_mut(),
    );
    if !(0..=u8::MAX.into()).contains(&code) {
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::from(code as u8)
}

#[cfg(not(target_os = "macos"))]
fn main() -> std::process::ExitCode {
    std::process::ExitCode::FAILURE
}
