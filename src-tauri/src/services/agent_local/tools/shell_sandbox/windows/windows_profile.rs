use windows_sys::Win32::Security::Isolation::{
    CreateAppContainerProfile, DeleteAppContainerProfile,
    DeriveAppContainerSidFromAppContainerName,
};
use windows_sys::Win32::Security::{FreeSid, PSID};

const PREFIX: &str = "Beaver.Shell.";

pub(super) struct Profile {
    name: String,
    sid: PSID,
    delete_on_drop: bool,
}

impl Profile {
    pub fn create() -> Result<Self, String> {
        let name = format!("{PREFIX}{}", uuid::Uuid::new_v4().simple());
        let wide = wide(&name);
        let mut sid = std::ptr::null_mut();
        // SAFETY: chaînes terminées par NUL et pointeur de sortie valide.
        let result = unsafe {
            CreateAppContainerProfile(
                wide.as_ptr(),
                wide.as_ptr(),
                wide.as_ptr(),
                std::ptr::null(),
                0,
                &mut sid,
            )
        };
        if result < 0 || sid.is_null() {
            return Err(super::error());
        }
        Ok(Self {
            name,
            sid,
            delete_on_drop: true,
        })
    }

    pub fn derive(name: &str) -> Result<Self, String> {
        if !valid_name(name) {
            return Err(super::error());
        }
        let wide = wide(name);
        let mut sid = std::ptr::null_mut();
        // SAFETY: nom validé, chaîne terminée par NUL, sortie valide.
        let result = unsafe { DeriveAppContainerSidFromAppContainerName(wide.as_ptr(), &mut sid) };
        if result < 0 || sid.is_null() {
            return Err(super::error());
        }
        Ok(Self {
            name: name.to_string(),
            sid,
            delete_on_drop: false,
        })
    }

    pub fn name(&self) -> &str { &self.name }
    pub fn sid(&self) -> PSID { self.sid }
    pub fn persist_for_cleanup(&mut self) { self.delete_on_drop = false; }
}

impl Drop for Profile {
    fn drop(&mut self) {
        if self.delete_on_drop {
            let _ = delete(&self.name);
        }
        // SAFETY: le SID est alloué par l'API AppContainer.
        unsafe { FreeSid(self.sid) };
    }
}

pub(super) fn delete(name: &str) -> Result<(), String> {
    if !valid_name(name) {
        return Err(super::error());
    }
    // SAFETY: nom validé et chaîne terminée par NUL.
    let result = unsafe { DeleteAppContainerProfile(wide(name).as_ptr()) };
    (result >= 0).then_some(()).ok_or_else(super::error)
}

pub(super) fn valid_name(name: &str) -> bool {
    name.len() == PREFIX.len() + 32
        && name.starts_with(PREFIX)
        && name[PREFIX.len()..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
