use super::super::CefUnavailableCategory;
use windows_sys::Win32::Foundation::{LocalFree, HLOCAL};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows_sys::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES};
use windows_sys::Win32::System::Memory::{FILE_MAP_READ, FILE_MAP_WRITE};

const MAX_SID_TEXT_UNITS: usize = 192;
const MAX_SDDL_UNITS: usize = 512;
const SYNCHRONIZE_ACCESS: u32 = 0x0010_0000;
const UNTRUSTED_MANDATORY_LABEL_SID: &str = "S-1-16-0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::services::browser) enum WindowsObjectKind {
    Mailbox,
    Control,
    AdmissionEvent,
    ClosingEvent,
}

impl WindowsObjectKind {
    #[cfg(test)]
    pub(in crate::services::browser) const ALL: [Self; 4] = [
        Self::Mailbox,
        Self::Control,
        Self::AdmissionEvent,
        Self::ClosingEvent,
    ];

    pub(in crate::services::browser) const fn helper_access(self) -> u32 {
        match self {
            Self::Mailbox => FILE_MAP_WRITE,
            Self::Control => FILE_MAP_READ,
            Self::AdmissionEvent | Self::ClosingEvent => SYNCHRONIZE_ACCESS,
        }
    }

    pub(super) const fn index(self) -> usize {
        match self {
            Self::Mailbox => 0,
            Self::Control => 1,
            Self::AdmissionEvent => 2,
            Self::ClosingEvent => 3,
        }
    }
}

pub(in crate::services::browser) struct WindowsObjectSecurity {
    descriptor: PSECURITY_DESCRIPTOR,
    attributes: SECURITY_ATTRIBUTES,
}

impl WindowsObjectSecurity {
    pub(in crate::services::browser) fn new(
        kind: WindowsObjectKind,
    ) -> Result<Self, CefUnavailableCategory> {
        let user = crate::services::private_store::current_windows_user()
            .map_err(|_| CefUnavailableCategory::Permission)?;
        let user_sid = sid_text(user.sid())?;
        let sddl = format!(
            "D:P(A;;GA;;;{user_sid})(A;;0x{:08x};;;RC)S:(ML;;NW;;;{UNTRUSTED_MANDATORY_LABEL_SID})",
            kind.helper_access(),
        );
        if sddl.encode_utf16().count() >= MAX_SDDL_UNITS || sddl.contains('\0') {
            return Err(CefUnavailableCategory::Permission);
        }
        let wide = sddl
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let mut descriptor = std::ptr::null_mut();
        let converted = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                wide.as_ptr(),
                SDDL_REVISION_1,
                &mut descriptor,
                std::ptr::null_mut(),
            )
        };
        if converted == 0 || descriptor.is_null() {
            return Err(CefUnavailableCategory::Permission);
        }
        let attributes = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor,
            bInheritHandle: 0,
        };
        Ok(Self {
            descriptor,
            attributes,
        })
    }

    pub(in crate::services::browser) fn attributes(&self) -> SECURITY_ATTRIBUTES {
        self.attributes
    }
}

impl Drop for WindowsObjectSecurity {
    fn drop(&mut self) {
        if !self.descriptor.is_null() {
            unsafe { LocalFree(self.descriptor.cast()) };
        }
    }
}

fn sid_text(sid: windows_sys::Win32::Security::PSID) -> Result<String, CefUnavailableCategory> {
    let mut text = std::ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(sid, &mut text) } == 0 || text.is_null() {
        return Err(CefUnavailableCategory::Permission);
    }
    let allocation = LocalText(text.cast());
    let length = (0..MAX_SID_TEXT_UNITS)
        .find(|index| unsafe { *text.add(*index) } == 0)
        .ok_or(CefUnavailableCategory::Permission)?;
    let value = String::from_utf16(unsafe { std::slice::from_raw_parts(text, length) })
        .map_err(|_| CefUnavailableCategory::Permission)?;
    drop(allocation);
    Ok(value)
}

struct LocalText(HLOCAL);

impl Drop for LocalText {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { LocalFree(self.0) };
        }
    }
}
