use super::super::{CefIpcNames, CefLaunchMarker, CefProcessRole, CefUnavailableCategory};
use super::{MacHelperObjects, MacPublicationObjects};

#[test]
fn helper_opens_every_descriptor_before_the_sandbox_and_observes_parent_events() {
    let root = crate::services::paths::data_dir().join("cef-ipc-test");
    let marker = CefLaunchMarker::generate(4, 9, CefProcessRole::Helper).expect("marker");
    let names = CefIpcNames::from_marker(&marker).expect("names");
    let parent = MacPublicationObjects::create(&root, &names, 9).expect("parent objects");
    let helper = MacHelperObjects::open(&root, &names).expect("helper objects");

    assert!(parent.descriptors_are_close_on_exec());
    assert!(helper.descriptors_are_close_on_exec());
    helper
        .publish(9, std::process::id(), 77_001, std::process::id())
        .expect("publication");
    assert_eq!(parent.mailbox_snapshot().expect("snapshot").generation, 9);
    parent.signal_admission();
    assert!(helper.admission_signaled().expect("admission"));
    parent.begin_closing(99_000).expect("closing");
    assert!(helper.closing_signaled().expect("closing event"));
    assert!(helper.control_snapshot().expect("control").closing);
}

#[test]
fn a_symlinked_runtime_root_is_refused() {
    use std::os::unix::fs::symlink;

    let base = crate::services::paths::data_dir().join("cef-ipc-symlink-test");
    let target = crate::services::paths::data_dir().join("cef-ipc-symlink-target");
    std::fs::create_dir_all(&target).expect("target");
    symlink(&target, &base).expect("symlink");
    let marker = CefLaunchMarker::generate(1, 1, CefProcessRole::Helper).expect("marker");
    let names = CefIpcNames::from_marker(&marker).expect("names");

    assert_eq!(
        MacPublicationObjects::create(&base, &names, 1).unwrap_err(),
        CefUnavailableCategory::Permission
    );
    std::fs::remove_file(base).expect("remove symlink");
}
