use super::emergency_actions_test_support::{ScriptedMacProcessActions, ScriptedProcess};
use super::emergency_slots_test_support::EmergencySlotsFixture;
use super::liveness_policy::MacLivenessDecision;
use super::process_state::{MacProcessObservation, MacSignalObservation, MacSignalResult};

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
fn normal_and_emergency_paths_share_the_original_closing_cap() {
    let mut fixture = EmergencySlotsFixture::new();
    let key = fixture.install_grouped_sleep();
    assert_eq!(
        fixture.slots.begin_closing_for_test(100 * MS, 200 * MS),
        Ok(())
    );
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
            200 * MS,
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

#[test]
fn later_drop_close_keeps_the_original_ultimate_cap() {
    let fixture = EmergencySlotsFixture::new();
    assert_eq!(
        fixture.slots.begin_closing_for_test(100 * MS, 200 * MS),
        Ok(())
    );
    assert_eq!(
        fixture.slots.begin_closing_for_test(50 * MS, 50 * MS),
        Ok(())
    );
    assert_eq!(
        fixture.slots.closing_deadlines_for_test(),
        Some((100 * MS, 200 * MS))
    );
}

#[test]
fn pending_unknown_never_sends_a_signal() {
    let mut fixture = EmergencySlotsFixture::new();
    let key = fixture.install_grouped_sleep();
    let actions = ScriptedMacProcessActions::single(key, [MacProcessObservation::Unknown], [], []);

    assert_eq!(
        fixture.slots.force_pass_with_for_test(&actions, 10 * MS),
        Ok(())
    );
    assert_eq!(actions.signal_count(key), 0);
    assert!(fixture.slots.has_entries());
}

#[test]
fn unknown_pre_signal_revalidation_never_sends_a_signal() {
    let mut fixture = EmergencySlotsFixture::new();
    let key = fixture.install_grouped_sleep();
    let actions = ScriptedMacProcessActions::single(
        key,
        [MacProcessObservation::Alive],
        [MacSignalObservation::Unknown],
        [],
    );

    assert_eq!(
        fixture.slots.force_pass_with_for_test(&actions, 10 * MS),
        Ok(())
    );
    assert_eq!(actions.signal_count(key), 0);
    assert!(fixture.slots.has_entries());
}

#[test]
fn final_pass_does_not_create_an_unknown_budget() {
    let mut fixture = EmergencySlotsFixture::new();
    let key = fixture.install_grouped_sleep();
    let actions = ScriptedMacProcessActions::single(
        key,
        [MacProcessObservation::Alive],
        [MacSignalObservation::Unknown],
        [],
    );

    assert_eq!(
        fixture
            .slots
            .force_final_pass_with_for_test(&actions, 10 * MS),
        Err(())
    );
    assert_eq!(actions.signal_count(key), 0);
    assert_eq!(
        fixture.slots.normal_observation_for_test(
            key.slot,
            key.generation,
            MacProcessObservation::Unknown,
            260 * MS,
        ),
        Ok(Some(MacLivenessDecision::Pending))
    );
}

#[test]
fn ready_identity_sends_once_and_remains_tracked_until_stopped() {
    let mut fixture = EmergencySlotsFixture::new();
    let key = fixture.install_grouped_sleep();
    let actions = ScriptedMacProcessActions::single(
        key,
        [MacProcessObservation::Alive, MacProcessObservation::Stopped],
        [MacSignalObservation::Ready],
        [Ok(MacSignalResult::Sent)],
    );

    assert_eq!(
        fixture.slots.force_pass_with_for_test(&actions, 10 * MS),
        Ok(())
    );
    assert_eq!(actions.signal_count(key), 1);
    assert!(fixture.slots.has_entries());
    assert_eq!(
        fixture.slots.force_pass_with_for_test(&actions, 20 * MS),
        Ok(())
    );
    assert!(!fixture.slots.has_entries());
}

#[test]
fn esrch_after_ready_clears_the_entry_without_failing_the_pass() {
    let mut fixture = EmergencySlotsFixture::new();
    let key = fixture.install_grouped_sleep();
    let actions = ScriptedMacProcessActions::single(
        key,
        [MacProcessObservation::Alive],
        [MacSignalObservation::Ready],
        [Ok(MacSignalResult::Stopped)],
    );

    assert_eq!(
        fixture.slots.force_pass_with_for_test(&actions, 10 * MS),
        Ok(())
    );
    assert_eq!(actions.signal_count(key), 1);
    assert!(!fixture.slots.has_entries());
}

#[test]
fn expired_unknown_fails_the_pass_but_does_not_skip_other_slots() {
    let mut fixture = EmergencySlotsFixture::new();
    let expiring = fixture.install_grouped_sleep();
    let ready = fixture.install_grouped_sleep();
    assert_eq!(
        fixture.slots.normal_observation_for_test(
            expiring.slot,
            expiring.generation,
            MacProcessObservation::Unknown,
            10 * MS,
        ),
        Ok(Some(MacLivenessDecision::Pending))
    );
    let actions = ScriptedMacProcessActions::new([
        (
            expiring,
            ScriptedProcess::observing([MacProcessObservation::Unknown]),
        ),
        (
            ready,
            ScriptedProcess::ready_and_signalling(Ok(MacSignalResult::Sent)),
        ),
    ]);

    assert_eq!(
        fixture.slots.force_pass_with_for_test(&actions, 260 * MS),
        Err(())
    );
    assert_eq!(actions.signal_count(ready), 1);
}
