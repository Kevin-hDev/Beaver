#[cfg(unix)]
use super::emergency::VerifiedProcessIdentity;
use super::emergency::{EmergencyInventory, EmergencyPublishError, EMERGENCY_CAPACITY};
#[cfg(unix)]
use super::emergency_drain::{EmergencyObservation, EmergencySignaler};
use super::emergency_registration::EmergencyHandoffReason;
use super::emergency_signaler::AppEmergencyPublisher;
#[cfg(unix)]
use super::emergency_signaler::NativeEmergencySignaler;
#[cfg(unix)]
use crate::services::owned_process::OwnedProcess;
#[cfg(unix)]
use crate::services::process_tree::ProcessKind;
#[cfg(unix)]
use std::process::{Command, Stdio};
#[cfg(target_os = "linux")]
use std::time::Duration;

#[test]
fn emergency_publisher_is_bounded_and_generation_guarded() {
    let publisher = AppEmergencyPublisher::new(EmergencyInventory::new());
    let registrations = (0..EMERGENCY_CAPACITY)
        .map(|index| {
            publisher
                .publish(
                    index as u32 + 2,
                    index as u64 + 1,
                    index as u64 + 9,
                    index as u128 + 1,
                )
                .expect("slot")
        })
        .collect::<Vec<_>>();
    assert!(publisher.publish(999, 1, 1, 1).is_err());
    assert_eq!(publisher.active_count_for_test(), EMERGENCY_CAPACITY);
    drop(registrations);
    assert_eq!(publisher.active_count_for_test(), 0);
}

#[test]
fn publisher_rejects_incomplete_executable_identity() {
    let publisher = AppEmergencyPublisher::new(EmergencyInventory::new());
    assert!(matches!(
        publisher.publish(2, 1, 1, 0),
        Err(EmergencyPublishError::InvalidIdentity)
    ));
}

#[cfg(unix)]
#[test]
fn native_signaler_rejects_zero_executable_identity() {
    let identity = VerifiedProcessIdentity::new(2, 1, 1).expect("numeric identity");
    assert_eq!(
        NativeEmergencySignaler.signal_or_recheck(identity, false),
        EmergencyObservation::IdentityMismatch
    );
}

#[cfg(target_os = "macos")]
#[test]
fn native_signaler_uses_the_owned_process_group() {
    let mut command = Command::new("/bin/sleep");
    command
        .arg("30")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = OwnedProcess::spawn(&mut command, ProcessKind::Ollama).expect("child");
    let identity = OwnedProcess::identity(child.id()).expect("identity");
    let emergency = VerifiedProcessIdentity::new_with_executable(
        identity.pid,
        identity.native_scope,
        identity.native_start_time,
        identity.executable,
    )
    .expect("identity");
    assert_eq!(
        NativeEmergencySignaler.signal_or_recheck(emergency, false),
        EmergencyObservation::Terminating
    );
    child.wait().expect("reap");
}

#[test]
fn watchdog_handoff_keeps_slot_until_generation_is_cleared() {
    let publisher = AppEmergencyPublisher::new(EmergencyInventory::new());
    let registration = publisher.publish(2, 1, 9, 1).expect("slot");
    let key = registration.key_for_test();
    registration.hand_off_to_watchdog(EmergencyHandoffReason::ReapFailed);
    assert_eq!(publisher.active_count_for_test(), 1);
    assert!(publisher.clear_for_test(key));
    assert_eq!(publisher.active_count_for_test(), 0);
}

#[cfg(unix)]
#[test]
fn native_signaler_refuses_an_identity_mismatch() {
    let mut command = Command::new("/bin/sleep");
    command
        .arg("30")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = OwnedProcess::spawn(&mut command, ProcessKind::Ollama).expect("child");
    let identity = OwnedProcess::identity(child.id()).expect("identity");
    let signaler = NativeEmergencySignaler;
    let wrong = VerifiedProcessIdentity::new(
        identity.pid,
        identity.native_scope,
        identity.native_start_time + 1,
    )
    .expect("wrong identity");
    assert_eq!(
        signaler.signal_or_recheck(wrong, false),
        EmergencyObservation::IdentityMismatch
    );
    crate::services::process_tree::terminate(&mut child, ProcessKind::Ollama);
}

#[cfg(target_os = "linux")]
#[test]
fn publisher_and_native_signaler_terminate_exact_child() {
    let mut command = Command::new("/bin/sleep");
    command
        .arg("30")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = OwnedProcess::spawn(&mut command, ProcessKind::Ollama).expect("child");
    let identity = OwnedProcess::identity(child.id()).expect("identity");
    let inventory = EmergencyInventory::new();
    let registration = inventory
        .try_publish(
            VerifiedProcessIdentity::new_with_executable(
                identity.pid,
                identity.native_scope,
                identity.native_start_time,
                identity.executable,
            )
            .expect("identity"),
        )
        .expect("registration");
    let signaler = NativeEmergencySignaler;
    inventory.drain_once(&signaler);
    std::thread::sleep(Duration::from_millis(20));
    let _ = child.wait();
    inventory.drain_once(&signaler);
    assert_eq!(inventory.active_count_for_test(), 0);
    drop(registration);
}
