use super::super::process::OllamaProcessError;
use super::super::spawn_profile::OllamaSpawnAttempt;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows_sys::Win32::System::Threading::{TerminateProcess, WaitForSingleObject};

pub(super) fn terminate_created_process(process: HANDLE, thread: HANDLE) {
    unsafe {
        TerminateProcess(process, 1);
        let first = WaitForSingleObject(process, 2_000);
        if first != WAIT_OBJECT_0 {
            let _ = WaitForSingleObject(process, u32::MAX);
        }
        CloseHandle(thread);
        CloseHandle(process);
    }
}

pub(in crate::services::ollama_manager) fn environment_block(
    attempt: &OllamaSpawnAttempt<'_>,
) -> Result<Vec<u16>, OllamaProcessError> {
    let mut block = Vec::new();
    for (key, value) in attempt.profile().environment().entries() {
        append_entry(&mut block, key, value)?;
    }
    let host = format!("127.0.0.1:{}", attempt.port());
    append_entry(&mut block, OsStr::new("OLLAMA_HOST"), OsStr::new(&host))?;
    block.push(0);
    Ok(block)
}

pub(in crate::services::ollama_manager) fn append_entry(
    block: &mut Vec<u16>,
    key: &OsStr,
    value: &OsStr,
) -> Result<(), OllamaProcessError> {
    let mut key_units = key.encode_wide();
    if key_units.clone().any(|unit| unit == 0) || value.encode_wide().any(|unit| unit == 0) {
        return Err(OllamaProcessError::InvalidState);
    }
    block.extend(key_units.by_ref());
    block.push('=' as u16);
    block.extend(value.encode_wide());
    block.push(0);
    Ok(())
}

pub(super) fn wide_path(path: &Path) -> Result<Vec<u16>, OllamaProcessError> {
    let mut value = path.as_os_str().encode_wide().collect::<Vec<_>>();
    if value.contains(&0) {
        return Err(OllamaProcessError::InvalidState);
    }
    value.push(0);
    Ok(value)
}

pub(super) fn wide_string(value: &str) -> Result<Vec<u16>, OllamaProcessError> {
    let mut result = value.encode_utf16().collect::<Vec<_>>();
    if result.contains(&0) {
        return Err(OllamaProcessError::InvalidState);
    }
    result.push(0);
    Ok(result)
}

pub(super) fn quote_path(path: &Path) -> String {
    let value = path.to_string_lossy();
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for character in value.chars() {
        if character == '"' {
            quoted.push('\\');
        }
        quoted.push(character);
    }
    quoted.push('"');
    quoted
}
