use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use sha2::{Digest, Sha256};
use windows_sys::Win32::Foundation::{
    CloseHandle, GENERIC_ALL, GENERIC_EXECUTE, GENERIC_READ, GENERIC_WRITE, HANDLE, LocalFree,
    WAIT_ABANDONED, WAIT_OBJECT_0,
};
use windows_sys::Win32::Security::Authorization::{
    EXPLICIT_ACCESS_W, GetNamedSecurityInfoW, REVOKE_ACCESS, SE_FILE_OBJECT,
    SET_ACCESS, SetEntriesInAclW, SetNamedSecurityInfoW, TRUSTEE_IS_SID,
    TRUSTEE_IS_UNKNOWN, TRUSTEE_W,
};
use windows_sys::Win32::Security::{
    ACL, CONTAINER_INHERIT_ACE, DACL_SECURITY_INFORMATION, OBJECT_INHERIT_ACE, PSID,
};
use windows_sys::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};

const ACL_MUTEX_TIMEOUT_MS: u32 = 30_000;

struct AclMutexGuard {
    handle: HANDLE,
}

pub(super) fn grant(path: &Path, sid: PSID, writable: bool) -> Result<(), String> {
    update(path, sid, Some(writable))
}

pub(super) fn revoke(path: &Path, sid: PSID) -> Result<(), String> {
    update(path, sid, None)
}

fn update(path: &Path, sid: PSID, writable: Option<bool>) -> Result<(), String> {
    if !path.exists() && writable.is_none() {
        return Ok(());
    }
    let _guard = lock_acl_updates()?;
    let wide = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect::<Vec<_>>();
    let mut old_acl: *mut ACL = std::ptr::null_mut();
    let mut descriptor = std::ptr::null_mut();
    // SAFETY: chemin NUL-terminé et sorties valides pour un objet fichier existant.
    let status = unsafe {
        GetNamedSecurityInfoW(
            wide.as_ptr(), SE_FILE_OBJECT, DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(), std::ptr::null_mut(), &mut old_acl,
            std::ptr::null_mut(), &mut descriptor,
        )
    };
    if status != 0 {
        return Err(super::error());
    }
    let trustee = TRUSTEE_W {
        pMultipleTrustee: std::ptr::null_mut(),
        MultipleTrusteeOperation: 0,
        TrusteeForm: TRUSTEE_IS_SID,
        TrusteeType: TRUSTEE_IS_UNKNOWN,
        ptstrName: sid.cast(),
    };
    let entry = EXPLICIT_ACCESS_W {
        grfAccessPermissions: match (writable, path.is_dir()) {
            (Some(true), true) => GENERIC_ALL,
            (Some(true), false) => GENERIC_READ | GENERIC_WRITE,
            (Some(false), true) => GENERIC_READ | GENERIC_EXECUTE,
            (Some(false), false) => GENERIC_READ,
            (None, _) => 0,
        },
        grfAccessMode: if writable.is_some() { SET_ACCESS } else { REVOKE_ACCESS },
        grfInheritance: if writable.is_some() && path.is_dir() {
            CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE
        } else {
            0
        },
        Trustee: trustee,
    };
    let mut new_acl = std::ptr::null_mut();
    // SAFETY: ACL et entrée restent valides pendant les deux appels.
    let update = unsafe { SetEntriesInAclW(1, &entry, old_acl, &mut new_acl) };
    let applied = if update == 0 {
        unsafe {
            SetNamedSecurityInfoW(
                wide.as_ptr() as *mut u16, SE_FILE_OBJECT, DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(), std::ptr::null_mut(), new_acl, std::ptr::null(),
            )
        }
    } else {
        update
    };
    // SAFETY: allocations retournées par les API de sécurité Windows.
    unsafe {
        if !new_acl.is_null() { LocalFree(new_acl.cast()); }
        if !descriptor.is_null() { LocalFree(descriptor); }
    }
    (applied == 0).then_some(()).ok_or_else(super::error)
}

fn lock_acl_updates() -> Result<AclMutexGuard, String> {
    let digest = Sha256::digest(
        crate::services::paths::data_dir()
            .to_string_lossy()
            .as_bytes(),
    );
    let suffix = digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let name = format!("Global\\Beaver.Shell.Acl.{suffix}")
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    // SAFETY: le nom est NUL-terminé et le handle reste possédé par le garde.
    let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
    if handle.is_null() {
        return Err(super::error());
    }
    // Un mutex abandonné transfère aussi sa propriété au processus courant.
    let wait = unsafe { WaitForSingleObject(handle, ACL_MUTEX_TIMEOUT_MS) };
    if wait != WAIT_OBJECT_0 && wait != WAIT_ABANDONED {
        unsafe { CloseHandle(handle) };
        return Err(super::error());
    }
    Ok(AclMutexGuard { handle })
}

impl Drop for AclMutexGuard {
    fn drop(&mut self) {
        // SAFETY: le garde possède le mutex et ferme exactement une fois son handle.
        unsafe {
            ReleaseMutex(self.handle);
            CloseHandle(self.handle);
        }
    }
}
