use super::{private_store_error, windows_acl, windows_token};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

static SECURITY_METADATA: Mutex<()> = Mutex::new(());

pub fn replace_file(source: &Path, destination: &Path) -> Result<(), String> {
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };
    let _metadata_guard = security_metadata_guard()?;
    let source = wide(source);
    let destination = wide(destination);
    let success = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    (success != 0).then_some(()).ok_or_else(private_store_error)
}

pub fn secure_acl(path: &Path) -> Result<(), String> {
    let _metadata_guard = security_metadata_guard()?;
    let user = windows_token::current_user()?;
    let path = wide(path);
    let is_directory = path_is_directory(path.as_slice())?;
    windows_acl::apply_and_verify(&path, user.sid(), is_directory)
}

fn security_metadata_guard() -> Result<MutexGuard<'static, ()>, String> {
    // Windows peut propager une ACL de dossier pendant le remplacement ou la
    // vérification d'un enfant ; une seule autorité ordonne ces métadonnées.
    SECURITY_METADATA.lock().map_err(|_| private_store_error())
}

fn path_is_directory(path: &[u16]) -> Result<bool, String> {
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileAttributesW, FILE_ATTRIBUTE_DIRECTORY, INVALID_FILE_ATTRIBUTES,
    };
    let attributes = unsafe { GetFileAttributesW(path.as_ptr()) };
    if attributes == INVALID_FILE_ATTRIBUTES {
        Err(private_store_error())
    } else {
        Ok(attributes & FILE_ATTRIBUTE_DIRECTORY != 0)
    }
}

fn wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(Some(0)).collect()
}
