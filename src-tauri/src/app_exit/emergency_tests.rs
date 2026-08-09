use super::emergency::{EmergencyInventory, VerifiedProcessIdentity, EMERGENCY_CAPACITY};
use super::emergency_drain::{EmergencyObservation, EmergencySignaler};
use std::sync::atomic::{AtomicUsize, Ordering};

struct GoneSignaler {
    calls: AtomicUsize,
}

impl EmergencySignaler for GoneSignaler {
    fn signal_or_recheck(
        &self,
        _identity: VerifiedProcessIdentity,
        _already_requested: bool,
    ) -> EmergencyObservation {
        self.calls.fetch_add(1, Ordering::AcqRel);
        EmergencyObservation::Exited
    }
}

fn identity(pid: u32) -> VerifiedProcessIdentity {
    VerifiedProcessIdentity::new(pid, pid as u64 + 10, pid as u64 + 100).expect("verified identity")
}

#[test]
fn invalid_process_identity_is_rejected() {
    assert!(VerifiedProcessIdentity::new(0, 1, 1).is_none());
    assert!(VerifiedProcessIdentity::new(1, 0, 1).is_none());
}

#[test]
fn emergency_inventory_is_fixed_and_bounded() {
    let inventory = EmergencyInventory::new();
    let registrations = (0..EMERGENCY_CAPACITY)
        .map(|index| {
            inventory
                .try_publish(identity(index as u32 + 1))
                .expect("emergency slot")
        })
        .collect::<Vec<_>>();

    assert!(inventory.try_publish(identity(999)).is_err());
    assert_eq!(inventory.active_count_for_test(), EMERGENCY_CAPACITY);
    drop(registrations);
    assert_eq!(inventory.active_count_for_test(), 0);
}

#[test]
fn stale_registration_cannot_clear_a_reused_slot() {
    let inventory = EmergencyInventory::new();
    let first = inventory.try_publish(identity(1)).expect("first slot");
    let stale = first.key_for_test();
    drop(first);
    let second = inventory.try_publish(identity(2)).expect("second slot");

    assert_ne!(stale, second.key_for_test());
    assert!(!inventory.clear_key_for_test(stale));
    assert_eq!(inventory.active_count_for_test(), 1);
}

#[test]
fn drain_only_uses_published_identity_and_clears_exited_process() {
    let inventory = EmergencyInventory::new();
    let registration = inventory.try_publish(identity(7)).expect("slot");
    let signaler = GoneSignaler {
        calls: AtomicUsize::new(0),
    };

    inventory.drain_once(&signaler);
    assert_eq!(signaler.calls.load(Ordering::Acquire), 1);
    assert_eq!(inventory.active_count_for_test(), 0);
    drop(registration);
}
