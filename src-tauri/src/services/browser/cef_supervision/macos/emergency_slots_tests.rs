use super::emergency_slots_test_support::EmergencySlotsFixture;
use super::liveness_policy::MacLivenessDecision;
use super::process_state::MacProcessObservation;

const MS: u64 = 1_000_000;

#[test]
fn normal_and_emergency_paths_share_one_unknown_budget() {
    let mut fixture = EmergencySlotsFixture::new();
    let key = fixture.install_grouped_sleep();
    assert_eq!(
        fixture.slots.normal_observation_for_test(
            key.slot,
            key.generation,
            MacProcessObservation::Unknown,
            10 * MS,
        ),
        Ok(Some(MacLivenessDecision::Pending))
    );
    assert_eq!(
        fixture.slots.emergency_observation_for_test(
            key.slot,
            key.generation,
            MacProcessObservation::Unknown,
            260 * MS,
        ),
        Ok(Some(MacLivenessDecision::Expired))
    );
}

#[test]
fn a_new_generation_starts_without_the_previous_unknown_budget() {
    let mut fixture = EmergencySlotsFixture::new();
    let old = fixture.install_grouped_sleep();
    assert_eq!(
        fixture.slots.normal_observation_for_test(
            old.slot,
            old.generation,
            MacProcessObservation::Unknown,
            10 * MS,
        ),
        Ok(Some(MacLivenessDecision::Pending))
    );
    fixture.slots.clear(old.slot, old.generation);
    let new = fixture.install_grouped_sleep();
    assert_eq!(new.slot, old.slot);
    assert_ne!(new.generation, old.generation);
    assert_eq!(
        fixture.slots.normal_observation_for_test(
            new.slot,
            new.generation,
            MacProcessObservation::Unknown,
            260 * MS,
        ),
        Ok(Some(MacLivenessDecision::Pending))
    );
}

#[test]
fn a_stale_active_key_cannot_touch_the_reused_slot() {
    let mut fixture = EmergencySlotsFixture::new();
    let old = fixture.install_grouped_sleep();
    fixture.slots.clear(old.slot, old.generation);
    let new = fixture.install_grouped_sleep();
    assert_eq!(
        fixture.slots.normal_observation_for_test(
            new.slot,
            new.generation,
            MacProcessObservation::Unknown,
            10 * MS,
        ),
        Ok(Some(MacLivenessDecision::Pending))
    );
    assert_eq!(
        fixture.slots.normal_observation_for_test(
            old.slot,
            old.generation,
            MacProcessObservation::Alive,
            20 * MS,
        ),
        Ok(None)
    );
    assert_eq!(
        fixture.slots.normal_observation_for_test(
            new.slot,
            new.generation,
            MacProcessObservation::Unknown,
            260 * MS,
        ),
        Ok(Some(MacLivenessDecision::Expired))
    );
}

#[test]
fn pending_unknown_keeps_the_admission_reserved() {
    let mut fixture = EmergencySlotsFixture::new();
    let key = fixture.install_grouped_sleep();
    assert!(fixture.slots.has_entries());
    assert_eq!(
        fixture.slots.normal_observation_for_test(
            key.slot,
            key.generation,
            MacProcessObservation::Unknown,
            10 * MS,
        ),
        Ok(Some(MacLivenessDecision::Pending))
    );
    assert!(fixture.slots.has_entries());
    assert_eq!(
        fixture.slots.normal_observation_for_test(
            key.slot,
            key.generation,
            MacProcessObservation::Stopped,
            20 * MS,
        ),
        Ok(Some(MacLivenessDecision::Stopped))
    );
    assert!(!fixture.slots.has_entries());
}
