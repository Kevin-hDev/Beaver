mod authority_slot;
#[cfg(windows)]
mod bootstrap;
mod constants;
mod diagnostics;
mod gate;
mod ipc_names;
#[cfg(target_os = "macos")]
mod macos;
mod mailbox;
mod reservation;
mod role_marker;
mod shared_layout;
mod slots;
#[cfg(windows)]
mod windows;

pub(super) use super::process_role::CefProcessRole;
#[cfg(windows)]
pub(crate) use bootstrap::WindowsHelperAdmission;
pub(crate) use constants::CEF_ADMISSION_SWITCH;
pub(super) use constants::CEF_SLOT_CAPACITY;
pub(super) use diagnostics::CefUnavailableCategory;
pub(super) use gate::CefLaunchGate;
pub(super) use ipc_names::CefIpcNames;
pub(super) use mailbox::CefPublication;
pub(super) use role_marker::{CefLaunchMarker, CefMarkerError};
pub(super) use shared_layout::{
    CefControlPage, CefEventPage, CefMailboxPage, CefSharedLayoutError,
};
pub(super) use slots::{CefAuthorityTable, CefTableError};
#[cfg(windows)]
pub(super) use windows::{WindowsCefTracker, WindowsCefTrackerHandle};

#[cfg(test)]
mod ipc_tests;
#[cfg(all(test, target_os = "macos"))]
#[path = "cef_supervision/macos/objects_tests.rs"]
mod macos_objects_tests;
#[cfg(test)]
mod protocol_tests;
#[cfg(test)]
mod table_tests;
#[cfg(all(test, windows))]
#[path = "cef_supervision/windows/identity_tests.rs"]
mod windows_identity_tests;
#[cfg(all(test, windows))]
#[path = "cef_supervision/windows/job_tests.rs"]
mod windows_job_tests;
#[cfg(all(test, windows))]
#[path = "cef_supervision/windows/objects_tests.rs"]
mod windows_objects_tests;
#[cfg(all(test, windows))]
#[path = "cef_supervision/windows/security_tests.rs"]
mod windows_security_tests;
#[cfg(all(test, windows))]
#[path = "cef_supervision/windows/tracker_tests.rs"]
mod windows_tracker_tests;
