mod authority_slot;
mod constants;
mod diagnostics;
mod gate;
mod mailbox;
mod reservation;
mod role_marker;
mod slots;

pub(super) use super::process_role::CefProcessRole;
pub(super) use constants::CEF_SLOT_CAPACITY;
pub(super) use diagnostics::CefUnavailableCategory;
pub(super) use gate::CefLaunchGate;
pub(super) use mailbox::CefPublication;
pub(super) use role_marker::{CefLaunchMarker, CefMarkerError};
pub(super) use slots::{CefAuthorityTable, CefTableError};

#[cfg(test)]
mod protocol_tests;
#[cfg(test)]
mod table_tests;
