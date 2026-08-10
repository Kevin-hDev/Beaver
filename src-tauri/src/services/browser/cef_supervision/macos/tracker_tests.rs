use super::macos::{MacCefTracker, MacHelperObjects, MacProcessIdentity};
use super::{CefIpcNames, CefUnavailableCategory};
use std::os::unix::process::CommandExt;
use std::time::{Duration, Instant};

#[test]
fn reaper_admits_only_the_stable_process_group_identity() {
    let root = crate::services::paths::data_dir()
        .join(format!("cef-mac-tracker-test-{}", std::process::id()));
    let mut child = grouped_sleep();
    let identity = MacProcessIdentity::read(child.id()).expect("child identity");
    let tracker = MacCefTracker::start(identity.test_executable(), root).expect("tracker");
    let ticket = tracker.handle().reserve().expect("reservation");
    let marker = ticket.decode_marker().expect("marker");
    let names = CefIpcNames::from_marker(&marker).expect("names");
    let helper = MacHelperObjects::open(
        &crate::services::paths::data_dir()
            .join(format!("cef-mac-tracker-test-{}", std::process::id())),
        &names,
    )
    .expect("helper objects");
    helper
        .publish(
            marker.generation(),
            identity.test_pid(),
            identity.test_started_at(),
            identity.test_process_group(),
        )
        .expect("publication");
    let deadline = Instant::now() + Duration::from_secs(2);
    while !helper.admission_signaled().expect("admission state") && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(helper.admission_signaled().expect("admitted"));
    assert_eq!(
        MacProcessIdentity::validate(
            identity.test_pid(),
            identity.test_parent_pid(),
            identity.test_started_at() + 1,
            identity.test_process_group(),
            identity.test_executable(),
        ),
        Err(CefUnavailableCategory::Admission)
    );
    drop(tracker);
    let deadline = Instant::now() + Duration::from_secs(2);
    while child.try_wait().expect("child status").is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(child.try_wait().expect("final status").is_some());
}

#[test]
fn closing_the_gate_preserves_an_admitted_helper_until_force_phase() {
    let root = crate::services::paths::data_dir()
        .join(format!("cef-mac-emergency-test-{}", std::process::id()));
    let mut child = grouped_sleep();
    let identity = MacProcessIdentity::read(child.id()).expect("child identity");
    let tracker = MacCefTracker::start(identity.test_executable(), root.clone()).expect("tracker");
    let ticket = tracker.handle().reserve().expect("reservation");
    let marker = ticket.decode_marker().expect("marker");
    let names = CefIpcNames::from_marker(&marker).expect("names");
    let helper = MacHelperObjects::open(&root, &names).expect("helper objects");
    helper
        .publish(
            marker.generation(),
            identity.test_pid(),
            identity.test_started_at(),
            identity.test_process_group(),
        )
        .expect("publication");
    wait_until_admitted(&helper);

    assert!(tracker.close_gate_for_test());
    std::thread::sleep(Duration::from_millis(50));
    assert!(child.try_wait().expect("child status").is_none());

    tracker.force_for_test();
    let deadline = Instant::now() + Duration::from_secs(2);
    while child.try_wait().expect("child status").is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(child.try_wait().expect("final status").is_some());
}

fn wait_until_admitted(helper: &MacHelperObjects) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while !helper.admission_signaled().expect("admission state") && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(helper.admission_signaled().expect("admitted"));
}

fn grouped_sleep() -> std::process::Child {
    let mut command = std::process::Command::new("/bin/sleep");
    command.arg("30");
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        });
    }
    command.spawn().expect("grouped child")
}
