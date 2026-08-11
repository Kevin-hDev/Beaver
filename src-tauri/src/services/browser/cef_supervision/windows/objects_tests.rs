use super::windows::{WindowsHelperObjects, WindowsPublicationObjects};
use super::{
    CefIpcNames, CefLaunchMarker, CefProcessRole, CefSharedLayoutError, CefUnavailableCategory,
};

#[test]
fn parent_and_helper_exchange_only_through_the_reserved_objects() {
    let marker = CefLaunchMarker::generate(5, 19, CefProcessRole::Helper).expect("marker");
    let names = CefIpcNames::from_marker(&marker).expect("names");
    let parent = WindowsPublicationObjects::create(&names, 19).expect("parent objects");
    let helper = WindowsHelperObjects::open(&names).expect("helper objects");

    assert!(parent.handles_are_non_inheritable());
    assert!(helper.handles_are_non_inheritable());
    helper
        .publish(19, std::process::id(), 88_001, 0)
        .expect("publication");
    let publication = parent.mailbox_snapshot().expect("snapshot");
    assert_eq!(publication.generation, 19);
    assert_eq!(publication.pid, std::process::id());

    parent.signal_admission().expect("admission signal");
    assert!(helper.wait_for_admission(100).expect("admission wait"));
    parent.begin_closing(91_000).expect("closing signal");
    assert!(helper.wait_for_closing(100).expect("closing wait"));
    let control = helper.control_snapshot().expect("control snapshot");
    assert!(control.closing);
    assert_eq!(control.deadline_ticks, 91_000);
}

#[test]
fn duplicate_native_names_fail_closed_without_replacing_live_objects() {
    let marker = CefLaunchMarker::generate(2, 3, CefProcessRole::Helper).expect("marker");
    let names = CefIpcNames::from_marker(&marker).expect("names");
    let first = WindowsPublicationObjects::create(&names, 3).expect("first objects");

    assert_eq!(
        WindowsPublicationObjects::create(&names, 3).unwrap_err(),
        CefUnavailableCategory::Object
    );
    assert_eq!(
        first.mailbox_snapshot(),
        Err(CefSharedLayoutError::Unpublished)
    );
}

#[test]
fn dropping_the_parent_removes_every_named_object() {
    let marker = CefLaunchMarker::generate(7, 4, CefProcessRole::Helper).expect("marker");
    let names = CefIpcNames::from_marker(&marker).expect("names");
    let parent = WindowsPublicationObjects::create(&names, 4).expect("parent objects");
    let helper = WindowsHelperObjects::open(&names).expect("helper objects");
    drop(helper);
    drop(parent);

    assert_eq!(
        WindowsHelperObjects::open(&names).unwrap_err(),
        CefUnavailableCategory::Object
    );
}
