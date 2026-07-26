use cef::{args::Args, *};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::io::Read;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

const BOOTSTRAP_SHA256: &str = "eab5d939293a666b210b8f5faec191324a017d6105485cfc45150863607bd367";
const MAX_BOOTSTRAP_BYTES: u64 = 32 * 1024 * 1024;
const MAX_APPLICATION_DLL_BYTES: u64 = 512 * 1024 * 1024;
const MAX_FORWARD_ARGS: usize = 64;
const MAX_ARG_UTF16: usize = 2_048;

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
    let bootstrap = checked_file(&root, "bootstrap.exe", MAX_BOOTSTRAP_BYTES)?;
    if file_sha256(&bootstrap, MAX_BOOTSTRAP_BYTES)? != BOOTSTRAP_SHA256 {
        return Err(());
    }
    let source_dll = checked_file(&root, "cl_go_dash_lib.dll", MAX_APPLICATION_DLL_BYTES)?;
    let module = root.join("bootstrap.dll");
    replace_module(&source_dll, &module, &root)?;
    let args = validated_forward_args()?;
    let status = std::process::Command::new(bootstrap)
        .args(args)
        .status()
        .map_err(|_| ())?;
    Ok(status.code().unwrap_or(1))
}

fn checked_file(root: &Path, name: &str, max_bytes: u64) -> Result<PathBuf, ()> {
    let path = root.join(name).canonicalize().map_err(|_| ())?;
    let metadata = path.symlink_metadata().map_err(|_| ())?;
    if !path.starts_with(root)
        || !metadata.file_type().is_file()
        || metadata.len() == 0
        || metadata.len() > max_bytes
    {
        return Err(());
    }
    Ok(path)
}

fn replace_module(source: &Path, destination: &Path, root: &Path) -> Result<(), ()> {
    if destination.parent() != Some(root) {
        return Err(());
    }
    let temporary = root.join("bootstrap.dll.tmp");
    let _ = std::fs::remove_file(&temporary);
    std::fs::copy(source, &temporary).map_err(|_| ())?;
    if destination.exists() {
        std::fs::remove_file(destination).map_err(|_| ())?;
    }
    std::fs::rename(temporary, destination).map_err(|_| ())
}

fn validated_forward_args() -> Result<Vec<OsString>, ()> {
    let mut result = Vec::with_capacity(MAX_FORWARD_ARGS);
    for argument in std::env::args_os().skip(1).take(MAX_FORWARD_ARGS + 1) {
        if result.len() == MAX_FORWARD_ARGS || argument.encode_wide().count() > MAX_ARG_UTF16 {
            return Err(());
        }
        result.push(argument);
    }
    Ok(result)
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
