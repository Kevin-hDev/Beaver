use cef::{args::Args, *};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

#[path = "windows_entry_plan.rs"]
mod plan;

const BOOTSTRAP_SHA256: &str = "eab5d939293a666b210b8f5faec191324a017d6105485cfc45150863607bd367";

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn RunWinMain(
    instance: cef::sys::HINSTANCE,
    _command_line: *const u8,
    _command_show: i32,
    sandbox_info: *mut u8,
) -> i32 {
    std::panic::catch_unwind(|| run_bootstrap_entry(instance, sandbox_info)).unwrap_or(1)
}

fn run_bootstrap_entry(instance: cef::sys::HINSTANCE, sandbox_info: *mut u8) -> i32 {
    // SAFETY: premier travail du point d'entrée DLL, avant CEF et tout thread.
    if !unsafe { crate::configure_git_network_policy() } {
        return 1;
    }
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = Args::from(MainArgs { instance });
    let result = execute_process(
        Some(args.as_main_args()),
        None::<&mut App>,
        sandbox_info.cast(),
    );
    if result >= 0 {
        return result;
    }
    if result != -1 || !crate::services::browser::windows_sandbox::register(sandbox_info) {
        return 1;
    }
    if !crate::prepare_browser_native_application() {
        return 1;
    }
    crate::run();
    0
}

pub(crate) fn launch_development_bootstrap() -> i32 {
    launch_development_bootstrap_inner().unwrap_or(1)
}

fn launch_development_bootstrap_inner() -> Result<i32, ()> {
    let executable = std::env::current_exe()
        .map_err(|_| ())?
        .canonicalize()
        .map_err(|_| ())?;
    let root = executable
        .parent()
        .ok_or(())?
        .canonicalize()
        .map_err(|_| ())?;
    let bootstrap = plan::checked_file(&root, "bootstrap.exe", plan::MAX_BOOTSTRAP_BYTES)?;
    if file_sha256(&bootstrap, plan::MAX_BOOTSTRAP_BYTES)? != BOOTSTRAP_SHA256 {
        return Err(());
    }
    plan::stage_application_module(&root)?;
    let development_bootstrap = plan::stage_bootstrap_executable(&root, &bootstrap)?;
    let args = plan::bootstrap_arguments(std::env::args_os().skip(1))?;
    let status = std::process::Command::new(development_bootstrap)
        .args(args)
        .status()
        .map_err(|_| ())?;
    Ok(status.code().unwrap_or(1))
}

fn file_sha256(path: &Path, max_bytes: u64) -> Result<String, ()> {
    let mut file = std::fs::File::open(path).map_err(|_| ())?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut total = 0_u64;
    loop {
        let read = file.read(&mut buffer).map_err(|_| ())?;
        if read == 0 {
            break;
        }
        total = total.checked_add(read as u64).ok_or(())?;
        if total > max_bytes {
            return Err(());
        }
        digest.update(&buffer[..read]);
    }
    Ok(hex::encode(digest.finalize()))
}
