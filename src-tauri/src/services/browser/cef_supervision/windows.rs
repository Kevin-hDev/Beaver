mod clock;
#[cfg(test)]
mod clock_tests;
mod confinement;
mod emergency_slots;
#[cfg(test)]
mod emergency_slots_tests;
mod handle;
mod helper_monitor;
#[cfg(test)]
mod helper_monitor_tests;
mod identity;
mod job;
mod mapping;
mod native_authority;
mod native_slot;
mod native_state;
mod objects;
mod process_query;
mod security;
mod tracker;
mod tracker_loop;
mod tracker_pending;
mod tracker_reservation;

#[cfg(test)]
pub(super) use confinement::WindowsConfinement;
pub(super) use helper_monitor::WindowsHelperMonitor;
pub(super) use identity::WindowsProcessIdentity;
#[cfg(test)]
pub(super) use identity::CEF_PROCESS_ACCESS_RIGHTS;
#[cfg(test)]
pub(super) use job::WindowsJobGuard;
#[cfg(test)]
pub(super) use native_authority::{
    classify_termination, WindowsNativeAuthority, WindowsTerminationState,
};
pub(super) use objects::WindowsHelperObjects;
#[cfg(test)]
pub(super) use objects::WindowsPublicationObjects;
pub(super) use process_query::WindowsProcessProbe;
#[cfg(test)]
pub(super) use security::{WindowsObjectKind, WindowsObjectSecurity};
pub(in crate::services::browser) use tracker::WindowsCefTracker;
#[cfg(not(feature = "windows-tests"))]
pub(in crate::services::browser) use tracker::WindowsCefTrackerHandle;
pub(in crate::services::browser) use tracker::WindowsTrackerShared;
