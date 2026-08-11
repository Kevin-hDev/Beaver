use super::{
    CefControlPage, CefEventPage, CefIpcNames, CefLaunchMarker, CefMailboxPage, CefProcessRole,
    CefSharedLayoutError,
};
use std::mem::{align_of, size_of};
use std::sync::atomic::Ordering;

#[test]
fn every_reservation_uses_four_distinct_bounded_names() {
    let marker = CefLaunchMarker::generate(3, 7, CefProcessRole::Helper).expect("marker");
    let names = CefIpcNames::from_marker(&marker).expect("names");
    let other_marker = CefLaunchMarker::generate(3, 7, CefProcessRole::Helper).expect("other");
    let other_names = CefIpcNames::from_marker(&other_marker).expect("other names");

    assert!(names.are_pairwise_distinct());
    assert!(names.max_name_bytes() <= 128);
    assert!(!names.constant_time_matches(&other_names));
    assert_eq!(format!("{names:?}"), "CefIpcNames([redacted])");
}

#[test]
fn mailbox_and_control_pages_have_fixed_cache_line_layouts() {
    assert_eq!(align_of::<CefMailboxPage>(), 64);
    assert_eq!(align_of::<CefControlPage>(), 64);
    assert_eq!(align_of::<CefEventPage>(), 64);
    assert!(size_of::<CefMailboxPage>() <= 128);
    assert!(size_of::<CefControlPage>() <= 128);
    assert!(size_of::<CefEventPage>() <= 128);
}

#[test]
fn event_pages_are_parent_signaled_and_fail_closed_on_invalid_schema() {
    let event = CefEventPage::new();
    assert!(!event.is_signaled().expect("initial event"));
    event.signal();
    assert!(event.is_signaled().expect("signaled event"));
    event.schema.store(0, Ordering::Release);
    assert_eq!(event.is_signaled(), Err(CefSharedLayoutError::Invalid));
}

#[test]
fn a_claimed_mailbox_snapshot_is_unchanged_by_later_corruption() {
    let mailbox = CefMailboxPage::new();
    mailbox.publish(8, 41, 123_456, 0).expect("publication");
    let sealed = mailbox.snapshot().expect("snapshot");

    mailbox.pid.store(999, Ordering::Release);
    mailbox.started_at.store(777, Ordering::Release);

    assert_eq!(sealed.generation, 8);
    assert_eq!(sealed.pid, 41);
    assert_eq!(sealed.started_at, 123_456);
    assert_eq!(sealed.native_group, 0);
}

#[test]
fn malformed_or_rewritten_mailbox_values_fail_closed() {
    let mailbox = CefMailboxPage::new();
    assert_eq!(
        mailbox.publish(0, 41, 1, 0),
        Err(CefSharedLayoutError::Invalid)
    );
    assert_eq!(
        mailbox.publish(1, 0, 1, 0),
        Err(CefSharedLayoutError::Invalid)
    );
    mailbox.publish(1, 41, 1, 0).expect("publication");
    assert_eq!(
        mailbox.publish(1, 42, 2, 0),
        Err(CefSharedLayoutError::AlreadyPublished)
    );
}

#[test]
fn control_page_exposes_only_the_current_generation_and_closing_deadline() {
    let control = CefControlPage::new(11).expect("control");
    assert_eq!(control.snapshot().expect("initial").generation, 11);
    assert!(!control.snapshot().expect("initial").closing);

    control.begin_closing(55_000).expect("closing");
    let closing = control.snapshot().expect("closing snapshot");
    assert!(closing.closing);
    assert_eq!(closing.deadline_ticks, 55_000);
    assert_eq!(control.begin_closing(0), Err(CefSharedLayoutError::Invalid));
}
