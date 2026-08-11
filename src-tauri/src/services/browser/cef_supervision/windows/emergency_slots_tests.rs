use super::emergency_slots::WindowsEmergencySlots;
use super::{WindowsHelperObjects, WindowsPublicationObjects};
use crate::services::browser::cef_supervision::{CefIpcNames, CefLaunchMarker, CefProcessRole};
use std::sync::Arc;

#[test]
fn emergency_closing_signals_pending_and_admitted_helpers() {
    let slots = Arc::new(WindowsEmergencySlots::new());
    let (pending_parent, pending_helper) = object_pair(3, 31);
    let (admitted_parent, admitted_helper) = object_pair(9, 47);
    let _pending = slots
        .install(3, 31, pending_parent)
        .expect("pending emergency registration");
    let _admitted = slots
        .install(9, 47, admitted_parent)
        .expect("admitted emergency registration");

    slots
        .begin_closing(88_000)
        .expect("signal every registered helper");

    assert!(pending_helper.wait_for_closing(100).expect("pending wait"));
    assert!(admitted_helper
        .wait_for_closing(100)
        .expect("admitted wait"));
    assert_deadline(&pending_helper, 88_000);
    assert_deadline(&admitted_helper, 88_000);
}

fn object_pair(
    slot: usize,
    generation: u64,
) -> (Arc<WindowsPublicationObjects>, WindowsHelperObjects) {
    let marker =
        CefLaunchMarker::generate(slot, generation, CefProcessRole::Helper).expect("marker");
    let names = CefIpcNames::from_marker(&marker).expect("names");
    let parent =
        Arc::new(WindowsPublicationObjects::create(&names, generation).expect("parent objects"));
    let helper = WindowsHelperObjects::open(&names).expect("helper objects");
    (parent, helper)
}

fn assert_deadline(helper: &WindowsHelperObjects, expected: u64) {
    let control = helper.control_snapshot().expect("control snapshot");
    assert!(control.closing);
    assert_eq!(control.deadline_ticks, expected);
}
