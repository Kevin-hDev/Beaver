use super::cef_cookie_gate_policy::cef_cookie_gate_policy_for_platform;

#[test]
fn runtime_policy_helper_is_only_compiled_with_the_cef_runtime() {
    let policy = include_str!("cef_cookie_gate_policy.rs");

    assert!(policy.contains(
        "#[cfg(any(target_os = \"macos\", target_os = \"windows\"))]\n\
         pub(super) fn cef_cookie_gate_policy()"
    ));
}

#[test]
fn security_gate_allows_time_for_system_approval() {
    let source = include_str!("cef_cookie_gate.rs");

    assert!(source.contains("Duration::from_secs(60)"));
}

#[test]
fn windows_cookie_gate_never_opens_chromiums_locked_cookie_store() {
    let cleanup = include_str!("cef_cookie_gate_cleanup.rs");
    let modules = include_str!("mod.rs");

    assert!(cleanup.contains("cef_cookie_gate_policy().verify_store_on_disk"));
    assert!(cleanup.contains("#[cfg(target_os = \"macos\")]"));
    assert!(modules.contains("#[cfg(any(test, target_os = \"macos\"))]\nmod cookie_store_probe;"));
}

#[test]
fn cookie_store_disk_verification_is_disabled_only_on_windows() {
    let windows = cef_cookie_gate_policy_for_platform(true);
    let macos = cef_cookie_gate_policy_for_platform(false);

    assert!(!windows.verify_store_on_disk);
    assert!(macos.verify_store_on_disk);
}
