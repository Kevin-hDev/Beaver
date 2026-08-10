use super::windows::{WindowsObjectKind, WindowsObjectSecurity};

#[test]
fn all_shared_objects_use_non_inheritable_handles_and_explicit_acl_and_mic() {
    for kind in WindowsObjectKind::ALL {
        let security = WindowsObjectSecurity::new(kind).expect("security descriptor");
        let attributes = security.attributes();
        assert_eq!(attributes.bInheritHandle, 0);
        assert!(!attributes.lpSecurityDescriptor.is_null());
        let sddl = WindowsObjectSecurity::sddl_for_test(kind).expect("security SDDL");
        assert!(sddl.starts_with("D:P(A;;GA;;;S-1-"));
        assert!(sddl.ends_with(&format!(
            ")(A;;0x{:08x};;;RC)S:(ML;;NW;;;S-1-16-0)",
            kind.helper_access()
        )));
    }
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
