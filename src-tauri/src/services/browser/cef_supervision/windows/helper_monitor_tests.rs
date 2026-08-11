use super::clock;
use super::helper_monitor::WindowsHelperMonitor;
use super::{WindowsHelperObjects, WindowsPublicationObjects};
use crate::services::browser::cef_supervision::{CefIpcNames, CefLaunchMarker, CefProcessRole};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[test]
fn helper_monitor_enforces_the_parent_deadline_once() {
    let (parent, helper) = object_pair(53);
    let terminations = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&terminations);
    let _monitor = WindowsHelperMonitor::start_with_action(helper, 53, move || {
        observed.fetch_add(1, Ordering::SeqCst);
    })
    .expect("monitor");
    let deadline = clock::ticks_at(Instant::now() + Duration::from_millis(20)).expect("deadline");

    parent.begin_closing(deadline).expect("closing signal");

    wait_for_action(&terminations);
    assert_eq!(terminations.load(Ordering::SeqCst), 1);
}

#[test]
fn helper_monitor_fails_closed_on_a_generation_mismatch() {
    let (_parent, helper) = object_pair(61);
    let terminations = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&terminations);
    let _monitor = WindowsHelperMonitor::start_with_action(helper, 62, move || {
        observed.fetch_add(1, Ordering::SeqCst);
    })
    .expect("monitor");

    wait_for_action(&terminations);
    assert_eq!(terminations.load(Ordering::SeqCst), 1);
}

fn object_pair(generation: u64) -> (WindowsPublicationObjects, Arc<WindowsHelperObjects>) {
    let marker = CefLaunchMarker::generate(17, generation, CefProcessRole::Helper).expect("marker");
    let names = CefIpcNames::from_marker(&marker).expect("names");
    let parent = WindowsPublicationObjects::create(&names, generation).expect("parent objects");
    let helper = Arc::new(WindowsHelperObjects::open(&names).expect("helper objects"));
    (parent, helper)
}

fn wait_for_action(terminations: &AtomicUsize) {
    let deadline = Instant::now() + Duration::from_secs(1);
    while terminations.load(Ordering::SeqCst) == 0 && Instant::now() < deadline {
        std::thread::yield_now();
    }
}
