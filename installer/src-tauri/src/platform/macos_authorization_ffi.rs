use super::{validate_call, AuthorizationScope, ProtectedTool};
use crate::error::InstallerError;
use std::ffi::{c_char, c_void, CString};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

type AuthorizationRef = *const c_void;

#[repr(C)]
struct AuthorizationItem {
    name: *const c_char,
    value_length: usize,
    value: *mut c_void,
    flags: u32,
}

#[repr(C)]
struct AuthorizationRights {
    count: u32,
    items: *mut AuthorizationItem,
}

#[link(name = "Security", kind = "framework")]
extern "C" {
    #[link_name = "AuthorizationCreate"]
    fn authorization_create(
        rights: *const AuthorizationRights,
        environment: *const AuthorizationRights,
        flags: u32,
        authorization: *mut AuthorizationRef,
    ) -> i32;
    #[link_name = "AuthorizationCopyRights"]
    fn authorization_copy_rights(
        authorization: AuthorizationRef,
        rights: *const AuthorizationRights,
        environment: *const AuthorizationRights,
        flags: u32,
        authorized_rights: *mut *mut AuthorizationRights,
    ) -> i32;
    #[link_name = "AuthorizationExecuteWithPrivileges"]
    fn authorization_execute(
        authorization: AuthorizationRef,
        tool: *const c_char,
        options: u32,
        arguments: *const *mut c_char,
        pipe: *mut *mut libc::FILE,
    ) -> i32;
    #[link_name = "AuthorizationFree"]
    fn authorization_free(authorization: AuthorizationRef, flags: u32) -> i32;
}

pub struct AuthorizationSession {
    reference: AuthorizationRef,
    scope: AuthorizationScope,
}

impl AuthorizationSession {
    pub fn new(scope: AuthorizationScope) -> Result<Self, InstallerError> {
        let mut reference = std::ptr::null();
        if unsafe { authorization_create(std::ptr::null(), std::ptr::null(), 0, &mut reference) }
            != 0
            || reference.is_null()
        {
            return Err(InstallerError::InstallFailed);
        }
        let right_name = CString::new("system.privilege.admin").expect("static right");
        let mut item = AuthorizationItem {
            name: right_name.as_ptr(),
            value_length: 0,
            value: std::ptr::null_mut(),
            flags: 0,
        };
        let rights = AuthorizationRights {
            count: 1,
            items: &mut item,
        };
        let status = unsafe {
            authorization_copy_rights(
                reference,
                &rights,
                std::ptr::null(),
                (1 << 0) | (1 << 1) | (1 << 4),
                std::ptr::null_mut(),
            )
        };
        if status != 0 {
            unsafe {
                authorization_free(reference, 1 << 3);
            }
            return Err(InstallerError::InstallFailed);
        }
        Ok(Self { reference, scope })
    }

    pub fn execute(
        &mut self,
        tool: ProtectedTool,
        arguments: &[PathBuf],
    ) -> Result<(), InstallerError> {
        validate_call(tool, arguments, &self.scope)?;
        let tool_path = CString::new(tool.path().as_os_str().as_bytes())
            .map_err(|_| InstallerError::InstallFailed)?;
        let mut values = Vec::with_capacity(arguments.len() + 1);
        if tool == ProtectedTool::Remove {
            values.push(CString::new("-rf").expect("static option"));
        }
        for argument in arguments {
            values.push(
                CString::new(argument.as_os_str().as_bytes())
                    .map_err(|_| InstallerError::InstallFailed)?,
            );
        }
        let mut pointers: Vec<*mut c_char> = values
            .iter_mut()
            .map(|value| value.as_ptr().cast_mut())
            .collect();
        pointers.push(std::ptr::null_mut());
        let mut pipe = std::ptr::null_mut();
        let status = unsafe {
            authorization_execute(
                self.reference,
                tool_path.as_ptr(),
                0,
                pointers.as_ptr(),
                &mut pipe,
            )
        };
        if status != 0 || pipe.is_null() {
            return Err(InstallerError::InstallFailed);
        }
        let mut buffer = [0_u8; 4096];
        while unsafe { libc::fread(buffer.as_mut_ptr().cast(), 1, buffer.len(), pipe) } > 0 {}
        let close_status = unsafe { libc::fclose(pipe) };
        (close_status == 0)
            .then_some(())
            .ok_or(InstallerError::InstallFailed)
    }
}

impl Drop for AuthorizationSession {
    fn drop(&mut self) {
        unsafe {
            authorization_free(self.reference, 1 << 3);
        }
    }
}

// ponytail: API Apple dépréciée conservée uniquement tant que Beaver n'a pas de helper signé.
