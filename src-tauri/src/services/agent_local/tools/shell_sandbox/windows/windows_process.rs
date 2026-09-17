use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Security::{
    CreateWellKnownSid, PSID, SECURITY_CAPABILITIES, SECURITY_MAX_SID_SIZE,
    SID_AND_ATTRIBUTES, WinCapabilityInternetClientServerSid, WinCapabilityInternetClientSid,
    WinCapabilityPrivateNetworkClientServerSid,
};
use windows_sys::Win32::System::Console::{
    GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
};
use windows_sys::Win32::System::Threading::*;

pub(super) fn run(executable: &Path, arguments: &[OsString], app_sid: PSID) -> Result<i32, String> {
    let types = [
        WinCapabilityInternetClientSid,
        WinCapabilityInternetClientServerSid,
        WinCapabilityPrivateNetworkClientServerSid,
    ];
    let mut sid_storage = [[0_u8; SECURITY_MAX_SID_SIZE as usize]; 3];
    let mut capabilities = Vec::with_capacity(types.len());
    for (kind, storage) in types.into_iter().zip(&mut sid_storage) {
        let mut size = storage.len() as u32;
        let ok = unsafe {
            CreateWellKnownSid(
                kind, std::ptr::null_mut(), storage.as_mut_ptr().cast(), &mut size,
            )
        };
        if ok == 0 { return Err(super::error()); }
        capabilities.push(SID_AND_ATTRIBUTES {
            Sid: storage.as_mut_ptr().cast(),
            Attributes: 4,
        });
    }
    let mut security = SECURITY_CAPABILITIES {
        AppContainerSid: app_sid,
        Capabilities: capabilities.as_mut_ptr(),
        CapabilityCount: capabilities.len() as u32,
        Reserved: 0,
    };
    let mut size = 0_usize;
    unsafe { InitializeProcThreadAttributeList(std::ptr::null_mut(), 1, 0, &mut size) };
    if size == 0 { return Err(super::error()); }
    let mut storage = vec![0_u8; size];
    let list = storage.as_mut_ptr().cast();
    if unsafe { InitializeProcThreadAttributeList(list, 1, 0, &mut size) } == 0 {
        return Err(super::error());
    }
    let updated = unsafe {
        UpdateProcThreadAttribute(
            list, 0, PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES as usize,
            (&mut security as *mut SECURITY_CAPABILITIES).cast(),
            std::mem::size_of::<SECURITY_CAPABILITIES>(),
            std::ptr::null_mut(), std::ptr::null(),
        )
    };
    if updated == 0 {
        unsafe { DeleteProcThreadAttributeList(list) };
        return Err(super::error());
    }
    let result = spawn_and_wait(executable, arguments, list);
    unsafe { DeleteProcThreadAttributeList(list) };
    result
}

fn spawn_and_wait(
    executable: &Path,
    arguments: &[OsString],
    list: LPPROC_THREAD_ATTRIBUTE_LIST,
) -> Result<i32, String> {
    let executable_wide = wide(executable.as_os_str());
    let mut command_line = command_line(executable.as_os_str(), arguments);
    let current_dir = wide(&std::env::current_dir().map_err(|_| super::error())?.into_os_string());
    let (stdin, stdout, stderr) = std_handles()?;
    let mut startup = STARTUPINFOEXW::default();
    startup.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdInput = stdin;
    startup.StartupInfo.hStdOutput = stdout;
    startup.StartupInfo.hStdError = stderr;
    startup.lpAttributeList = list;
    let mut process = PROCESS_INFORMATION::default();
    let created = unsafe {
        CreateProcessW(
            executable_wide.as_ptr(), command_line.as_mut_ptr(), std::ptr::null(),
            std::ptr::null(), 1, EXTENDED_STARTUPINFO_PRESENT, std::ptr::null(),
            current_dir.as_ptr(), &startup.StartupInfo, &mut process,
        )
    };
    if created == 0 { return Err(super::error()); }
    unsafe { CloseHandle(process.hThread) };
    let waited = unsafe { WaitForSingleObject(process.hProcess, INFINITE) };
    let mut exit_code = 1_u32;
    let read = unsafe { GetExitCodeProcess(process.hProcess, &mut exit_code) };
    unsafe { CloseHandle(process.hProcess) };
    if waited != 0 || read == 0 { return Err(super::error()); }
    Ok(i32::try_from(exit_code).unwrap_or(1))
}

fn std_handles() -> Result<(HANDLE, HANDLE, HANDLE), String> {
    let handles = unsafe {
        (GetStdHandle(STD_INPUT_HANDLE), GetStdHandle(STD_OUTPUT_HANDLE), GetStdHandle(STD_ERROR_HANDLE))
    };
    if [handles.0, handles.1, handles.2]
        .into_iter()
        .any(|handle| handle.is_null() || handle == INVALID_HANDLE_VALUE)
    {
        Err(super::error())
    } else {
        Ok(handles)
    }
}

fn command_line(executable: &OsStr, arguments: &[OsString]) -> Vec<u16> {
    let mut values = Vec::with_capacity(arguments.len() + 1);
    values.push(executable.to_os_string());
    values.extend_from_slice(arguments);
    let text = values.iter().map(|value| quote(value)).collect::<Vec<_>>().join(" ");
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn quote(value: &OsStr) -> String {
    let value = value.to_string_lossy();
    let mut result = String::from("\"");
    let mut slashes = 0;
    for character in value.chars() {
        if character == '\\' { slashes += 1; continue; }
        if character == '"' { result.push_str(&"\\".repeat(slashes * 2 + 1)); }
        else { result.push_str(&"\\".repeat(slashes)); }
        slashes = 0;
        result.push(character);
    }
    result.push_str(&"\\".repeat(slashes * 2));
    result.push('"');
    result
}

fn wide(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(std::iter::once(0)).collect()
}
