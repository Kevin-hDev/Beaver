use super::super::{CefLaunchMarker, CefMarkerError};
use std::fmt;
use zeroize::Zeroizing;

pub(in crate::services::browser) struct WindowsLaunchTicket {
    marker: Zeroizing<String>,
}

impl WindowsLaunchTicket {
    pub(super) fn new(marker: &CefLaunchMarker) -> Self {
        Self {
            marker: marker.encode(),
        }
    }

    pub(in crate::services::browser) fn encoded_marker(&self) -> &str {
        &self.marker
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn decode_marker(
        &self,
    ) -> Result<CefLaunchMarker, CefMarkerError> {
        CefLaunchMarker::decode_unique(&[&self.marker])
    }
}

impl fmt::Debug for WindowsLaunchTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("WindowsLaunchTicket([redacted])")
    }
}
