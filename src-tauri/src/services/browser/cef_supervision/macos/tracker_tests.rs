use super::macos::{
    helper_parent_changed_for_test, MacCefTracker, MacHelperObjects, MacProcessIdentity,
};
use super::{CefIpcNames, CefUnavailableCategory};
use std::os::unix::process::CommandExt;
use std::time::{Duration, Instant};

#[test]
fn helper_accepts_only_its_validated_current_parent() {
    let current = unsafe { libc::getppid() };
    assert!(current > 0);
    assert!(!helper_parent_changed_for_test(current as u32));
    assert!(helper_parent_changed_for_test(0));
    assert!(helper_parent_changed_for_test(
        (current as u32).saturating_add(1)
    ));
}

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
    let mut allowed_executables =
        std::array::from_fn(|index| std::path::PathBuf::from(format!("/invalid/{index}")));
    allowed_executables[4] = identity.test_executable().to_path_buf();
    assert!(MacProcessIdentity::validate(
        identity.test_pid(),
        identity.test_parent_pid(),
        identity.test_started_at(),
        identity.test_process_group(),
        &allowed_executables,
    )
    .is_ok());
    assert_eq!(
        MacProcessIdentity::validate(
            identity.test_pid(),
            identity.test_parent_pid(),
            identity.test_started_at() + 1,
            identity.test_process_group(),
            &std::array::from_fn(|_| identity.test_executable().to_path_buf()),
        ),
        Err(CefUnavailableCategory::Admission)
    );
    child.kill().expect("stop child");
    let deadline = Instant::now() + Duration::from_secs(2);
    while tracker.has_runnable_for_test()
        && tracker.failure_for_test().is_none()
        && Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(tracker.failure_for_test(), None);
    assert!(!tracker.has_runnable_for_test());
    drop(tracker);
    let deadline = Instant::now() + Duration::from_secs(2);
    while child.try_wait().expect("child status").is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(child.try_wait().expect("final status").is_some());
}

#[test]
fn reaper_treats_an_unreaped_exited_helper_as_stopped() {
    let mut child = grouped_sleep();
    let identity = MacProcessIdentity::read(child.id()).expect("child identity");
    child.kill().expect("kill child");
    let deadline = Instant::now() + Duration::from_secs(2);
    let observed = loop {
        let current = identity.test_is_alive();
        if current == Ok(false) || Instant::now() >= deadline {
            break current;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    child.wait().expect("reap child");

    assert_eq!(observed, Ok(false));
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

#[test]
fn emergency_reaper_survives_the_normal_tracker_stopping() {
    let root = crate::services::paths::data_dir().join(format!(
        "cef-mac-independent-reaper-test-{}",
        std::process::id()
    ));
    let mut child = grouped_sleep();
    let identity = MacProcessIdentity::read(child.id()).expect("child identity");
    let mut tracker =
        MacCefTracker::start(identity.test_executable(), root.clone()).expect("tracker");
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

    tracker.stop_normal_for_test();
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
