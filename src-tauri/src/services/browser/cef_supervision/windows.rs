mod confinement;
mod handle;
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

#[cfg(test)]
pub(super) use confinement::WindowsConfinement;
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
pub(in crate::services::browser) use tracker::WindowsTrackerShared;
pub(in crate::services::browser) use tracker::{WindowsCefTracker, WindowsCefTrackerHandle};
