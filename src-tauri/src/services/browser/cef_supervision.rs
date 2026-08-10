mod authority_slot;
#[cfg(windows)]
mod bootstrap;
mod constants;
mod diagnostics;
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub(super) mod emergency;
mod gate;
mod ipc_names;
#[cfg(any(target_os = "windows", target_os = "macos"))]
mod launch_ticket;
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
#[cfg(all(windows, not(feature = "windows-tests")))]
pub(crate) use bootstrap::WindowsHelperAdmission;
#[cfg(native_browser)]
pub(crate) use constants::CEF_ADMISSION_SWITCH;
#[cfg(test)]
pub(super) use constants::CEF_SLOT_CAPACITY;
pub(super) use diagnostics::CefUnavailableCategory;
#[cfg(test)]
pub(super) use gate::CefLaunchGate;
pub(super) use ipc_names::CefIpcNames;
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub(super) use launch_ticket::CefLaunchTicket;
#[cfg(target_os = "macos")]
pub(super) use macos::{parse_helper_marker, MacHelperBootstrap};
#[cfg(target_os = "macos")]
pub(super) use macos::{MacCefTracker, MacCefTrackerHandle};
pub(super) use mailbox::CefPublication;
pub(super) use role_marker::{CefLaunchMarker, CefMarkerError};
#[cfg(any(test, target_os = "macos"))]
pub(super) use shared_layout::CefEventPage;
pub(super) use shared_layout::{CefControlPage, CefMailboxPage, CefSharedLayoutError};
pub(super) use slots::{CefAuthorityTable, CefTableError};
#[cfg(windows)]
pub(super) use windows::{WindowsCefTracker, WindowsCefTrackerHandle};

#[cfg(test)]
mod capability_tests;
#[cfg(test)]
mod ipc_tests;
#[cfg(all(test, target_os = "macos"))]
#[path = "cef_supervision/macos/objects_tests.rs"]
mod macos_objects_tests;
#[cfg(all(test, target_os = "macos"))]
#[path = "cef_supervision/macos/tracker_tests.rs"]
mod macos_tracker_tests;
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
