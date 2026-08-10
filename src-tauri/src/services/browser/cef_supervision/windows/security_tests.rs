use super::windows::{WindowsObjectKind, WindowsObjectSecurity};
use windows_sys::Win32::Foundation::GENERIC_ALL;
use windows_sys::Win32::Security::{
    CreateWellKnownSid, EqualSid, GetAce, GetSecurityDescriptorDacl, GetSecurityDescriptorSacl,
    WinRestrictedCodeSid, WinUntrustedLabelSid, ACCESS_ALLOWED_ACE, ACL, SECURITY_MAX_SID_SIZE,
    SYSTEM_MANDATORY_LABEL_ACE,
};
use windows_sys::Win32::System::SystemServices::SYSTEM_MANDATORY_LABEL_NO_WRITE_UP;

#[test]
fn all_shared_objects_use_non_inheritable_handles_and_explicit_acl_and_mic() {
    for kind in WindowsObjectKind::ALL {
        let security = WindowsObjectSecurity::new(kind).expect("security descriptor");
        let attributes = security.attributes();
        assert_eq!(attributes.bInheritHandle, 0);

        let mut dacl_present = 0;
        let mut dacl: *mut ACL = std::ptr::null_mut();
        let mut dacl_defaulted = 0;
        let dacl_ok = unsafe {
            GetSecurityDescriptorDacl(
                attributes.lpSecurityDescriptor,
                &mut dacl_present,
                &mut dacl,
                &mut dacl_defaulted,
            )
        };
        assert_ne!(dacl_ok, 0);
        assert_ne!(dacl_present, 0);
        assert!(!dacl.is_null());
        assert_eq!(dacl_defaulted, 0);
        let dacl = unsafe { &*dacl };
        assert_eq!(dacl.AceCount, 2);
        let user_ace = allowed_ace(dacl, 0);
        let restricted_ace = allowed_ace(dacl, 1);
        assert_eq!(user_ace.Mask, GENERIC_ALL);
        assert_eq!(restricted_ace.Mask, kind.helper_access());
        let user = crate::services::private_store::current_windows_user().expect("current user");
        assert_ne!(unsafe { EqualSid(ace_sid(user_ace), user.sid()) }, 0);
        let restricted_sid = well_known_sid(WinRestrictedCodeSid);
        assert_ne!(
            unsafe {
                EqualSid(
                    ace_sid(restricted_ace),
                    restricted_sid.as_ptr().cast_mut().cast(),
                )
            },
            0
        );

        let mut sacl_present = 0;
        let mut sacl: *mut ACL = std::ptr::null_mut();
        let mut sacl_defaulted = 0;
        let sacl_ok = unsafe {
            GetSecurityDescriptorSacl(
                attributes.lpSecurityDescriptor,
                &mut sacl_present,
                &mut sacl,
                &mut sacl_defaulted,
            )
        };
        assert_ne!(sacl_ok, 0);
        assert_ne!(sacl_present, 0);
        assert!(!sacl.is_null());
        assert_eq!(sacl_defaulted, 0);
        let sacl = unsafe { &*sacl };
        assert_eq!(sacl.AceCount, 1);
        let label = mandatory_label(sacl, 0);
        assert_eq!(label.Mask, SYSTEM_MANDATORY_LABEL_NO_WRITE_UP);
        let untrusted_sid = well_known_sid(WinUntrustedLabelSid);
        assert_ne!(
            unsafe { EqualSid(label_sid(label), untrusted_sid.as_ptr().cast_mut().cast()) },
            0
        );
    }
}

fn allowed_ace(acl: &ACL, index: u32) -> &ACCESS_ALLOWED_ACE {
    let mut raw = std::ptr::null_mut();
    assert_ne!(unsafe { GetAce(acl, index, &mut raw) }, 0);
    assert!(!raw.is_null());
    unsafe { &*raw.cast::<ACCESS_ALLOWED_ACE>() }
}

fn mandatory_label(acl: &ACL, index: u32) -> &SYSTEM_MANDATORY_LABEL_ACE {
    let mut raw = std::ptr::null_mut();
    assert_ne!(unsafe { GetAce(acl, index, &mut raw) }, 0);
    assert!(!raw.is_null());
    unsafe { &*raw.cast::<SYSTEM_MANDATORY_LABEL_ACE>() }
}

fn ace_sid(ace: &ACCESS_ALLOWED_ACE) -> windows_sys::Win32::Security::PSID {
    std::ptr::addr_of!(ace.SidStart).cast_mut().cast()
}

fn label_sid(ace: &SYSTEM_MANDATORY_LABEL_ACE) -> windows_sys::Win32::Security::PSID {
    std::ptr::addr_of!(ace.SidStart).cast_mut().cast()
}

fn well_known_sid(kind: i32) -> [u8; SECURITY_MAX_SID_SIZE as usize] {
    let mut sid = [0_u8; SECURITY_MAX_SID_SIZE as usize];
    let mut size = SECURITY_MAX_SID_SIZE;
    assert_ne!(
        unsafe {
            CreateWellKnownSid(
                kind,
                std::ptr::null_mut(),
                sid.as_mut_ptr().cast(),
                &mut size,
            )
        },
        0
    );
    sid
}

#[test]
fn restricted_helper_rights_are_minimal_for_each_object_kind() {
    assert_eq!(WindowsObjectKind::Mailbox.helper_access(), 0x0002);
    assert_eq!(WindowsObjectKind::Control.helper_access(), 0x0004);
    assert_eq!(
        WindowsObjectKind::AdmissionEvent.helper_access(),
        0x0010_0000
    );
    assert_eq!(WindowsObjectKind::ClosingEvent.helper_access(), 0x0010_0000);
}
