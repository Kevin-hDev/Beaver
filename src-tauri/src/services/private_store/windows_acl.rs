use windows_sys::core::PWSTR;
use windows_sys::Win32::Foundation::{LocalFree, ERROR_SUCCESS, HLOCAL};
use windows_sys::Win32::Security::Authorization::{
    GetExplicitEntriesFromAclW, GetNamedSecurityInfoW, SetEntriesInAclW, SetNamedSecurityInfoW,
    EXPLICIT_ACCESS_W, GRANT_ACCESS, NO_MULTIPLE_TRUSTEE, SET_ACCESS, SE_FILE_OBJECT,
    TRUSTEE_IS_SID, TRUSTEE_IS_UNKNOWN, TRUSTEE_IS_USER, TRUSTEE_W,
};
use windows_sys::Win32::Security::{
    EqualSid, GetSecurityDescriptorControl, IsValidSid, ACL, DACL_SECURITY_INFORMATION,
    NO_INHERITANCE, PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID,
    SE_DACL_PROTECTED, SUB_CONTAINERS_AND_OBJECTS_INHERIT,
};
use windows_sys::Win32::Storage::FileSystem::FILE_ALL_ACCESS;

const ERROR: &str = "stockage privé indisponible";

pub fn apply_and_verify(path: &[u16], sid: PSID, is_directory: bool) -> Result<(), String> {
    let inheritance = inheritance_for(is_directory);
    let entry = explicit_access(sid, inheritance);
    let mut acl = std::ptr::null_mut();
    let status = unsafe { SetEntriesInAclW(1, &entry, std::ptr::null(), &mut acl) };
    if status != ERROR_SUCCESS || acl.is_null() {
        return Err(ERROR.to_string());
    }
    let acl_guard = LocalAllocation(acl.cast());
    let status = unsafe {
        SetNamedSecurityInfoW(
            path.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            acl,
            std::ptr::null(),
        )
    };
    if status != ERROR_SUCCESS {
        return Err(ERROR.to_string());
    }
    drop(acl_guard);
    verify(path, sid, inheritance)
}

fn verify(path: &[u16], sid: PSID, inheritance: u32) -> Result<(), String> {
    let mut acl: *mut ACL = std::ptr::null_mut();
    let mut descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
    let status = unsafe {
        GetNamedSecurityInfoW(
            path.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut acl,
            std::ptr::null_mut(),
            &mut descriptor,
        )
    };
    if status != ERROR_SUCCESS || acl.is_null() || descriptor.is_null() {
        return Err(ERROR.to_string());
    }
    let descriptor_guard = LocalAllocation(descriptor.cast());
    verify_descriptor(descriptor)?;
    verify_entries(acl, sid, inheritance)?;
    drop(descriptor_guard);
    Ok(())
}

fn verify_descriptor(descriptor: PSECURITY_DESCRIPTOR) -> Result<(), String> {
    let mut control = 0_u16;
    let mut revision = 0_u32;
    let success = unsafe { GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) };
    if success == 0 || control & SE_DACL_PROTECTED == 0 {
        Err(ERROR.to_string())
    } else {
        Ok(())
    }
}

fn verify_entries(acl: *const ACL, sid: PSID, inheritance: u32) -> Result<(), String> {
    let mut count = 0_u32;
    let mut entries = std::ptr::null_mut();
    let status = unsafe { GetExplicitEntriesFromAclW(acl, &mut count, &mut entries) };
    if status != ERROR_SUCCESS || count != 1 || entries.is_null() {
        return Err(ERROR.to_string());
    }
    let entries_guard = LocalAllocation(entries.cast());
    let entry = unsafe { &*entries };
    let entry_sid: PSID = entry.Trustee.ptstrName.cast();
    let valid_mode = matches!(entry.grfAccessMode, GRANT_ACCESS | SET_ACCESS);
    let valid = valid_mode
        && entry.grfAccessPermissions & FILE_ALL_ACCESS == FILE_ALL_ACCESS
        && entry.grfInheritance == inheritance
        && entry.Trustee.TrusteeForm == TRUSTEE_IS_SID
        && valid_trustee_type(entry.Trustee.TrusteeType)
        && !entry_sid.is_null()
        && unsafe { IsValidSid(entry_sid) } != 0
        && unsafe { EqualSid(entry_sid, sid) } != 0;
    drop(entries_guard);
    valid.then_some(()).ok_or_else(|| ERROR.to_string())
}

fn valid_trustee_type(trustee_type: i32) -> bool {
    matches!(trustee_type, TRUSTEE_IS_UNKNOWN | TRUSTEE_IS_USER)
}

fn explicit_access(sid: PSID, inheritance: u32) -> EXPLICIT_ACCESS_W {
    EXPLICIT_ACCESS_W {
        grfAccessPermissions: FILE_ALL_ACCESS,
        grfAccessMode: SET_ACCESS,
        grfInheritance: inheritance,
        Trustee: TRUSTEE_W {
            pMultipleTrustee: std::ptr::null_mut(),
            MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_USER,
            ptstrName: sid.cast::<u16>() as PWSTR,
        },
    }
}

fn inheritance_for(is_directory: bool) -> u32 {
    if is_directory {
        SUB_CONTAINERS_AND_OBJECTS_INHERIT
    } else {
        NO_INHERITANCE
    }
}

struct LocalAllocation(HLOCAL);

impl Drop for LocalAllocation {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { LocalFree(self.0) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_user_and_unknown_trustee_types() {
        assert!(valid_trustee_type(TRUSTEE_IS_USER));
        assert!(valid_trustee_type(TRUSTEE_IS_UNKNOWN));
        assert!(!valid_trustee_type(TRUSTEE_IS_USER + 1));
    }
}
