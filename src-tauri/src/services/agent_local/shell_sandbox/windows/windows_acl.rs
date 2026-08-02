use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows_sys::Win32::Foundation::{
    GENERIC_ALL, GENERIC_EXECUTE, GENERIC_READ, LocalFree,
};
use windows_sys::Win32::Security::Authorization::{
    EXPLICIT_ACCESS_W, GetNamedSecurityInfoW, REVOKE_ACCESS, SE_FILE_OBJECT,
    SET_ACCESS, SetEntriesInAclW, SetNamedSecurityInfoW, TRUSTEE_IS_SID,
    TRUSTEE_IS_UNKNOWN, TRUSTEE_W,
};
use windows_sys::Win32::Security::{
    ACL, CONTAINER_INHERIT_ACE, DACL_SECURITY_INFORMATION, OBJECT_INHERIT_ACE, PSID,
};

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
        grfAccessPermissions: match writable {
            Some(true) => GENERIC_ALL,
            Some(false) => GENERIC_READ | GENERIC_EXECUTE,
            None => 0,
        },
        grfAccessMode: if writable.is_some() { SET_ACCESS } else { REVOKE_ACCESS },
        grfInheritance: if writable.is_some() {
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
