mod arguments;
mod bootstrap;
mod clock;
mod emergency_slots;
mod identity;
mod liveness_policy;
mod mapping;
mod objects;
mod pending;
mod process_state;
mod reaper;
mod tracker;
mod tracker_lifecycle;
mod tracker_loop;
mod tracker_reservation;

pub(in crate::services::browser) use arguments::parse_helper_marker;
#[cfg(test)]
pub(super) fn helper_parent_changed_for_test(expected: u32) -> bool {
    bootstrap::parent_changed(expected)
}
pub(in crate::services::browser) use bootstrap::MacHelperBootstrap;
#[cfg(test)]
pub(super) use identity::MacProcessIdentity;
pub(super) use objects::{MacHelperObjects, MacPublicationObjects};
pub(in crate::services::browser) use tracker::MacTrackerShared;
pub(in crate::services::browser) use tracker::{MacCefTracker, MacCefTrackerHandle};

#[cfg(test)]
mod liveness_policy_tests;
#[cfg(test)]
mod objects_tests;
#[cfg(test)]
mod tracker_tests;
