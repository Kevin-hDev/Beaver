mod constants;
mod diagnostics;
mod role_marker;

pub(super) use super::process_role::CefProcessRole;
pub(super) use constants::CEF_SLOT_CAPACITY;
pub(super) use diagnostics::CefUnavailableCategory;
pub(super) use role_marker::{CefLaunchMarker, CefMarkerError};

#[cfg(test)]
mod protocol_tests;
