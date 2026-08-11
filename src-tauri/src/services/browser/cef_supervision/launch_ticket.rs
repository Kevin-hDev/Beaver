use super::constants::CEF_MARKER_MAX_BYTES;
use super::CefLaunchMarker;
use std::fmt;
use zeroize::Zeroizing;

pub(in crate::services::browser) struct CefLaunchTicket {
    marker: Zeroizing<String>,
}

impl CefLaunchTicket {
    pub(super) fn new(marker: &CefLaunchMarker) -> Self {
        Self {
            marker: marker.encode(),
        }
    }

    pub(in crate::services::browser) fn encoded_marker(&self) -> &str {
        &self.marker
    }

    pub(in crate::services::browser) fn constant_time_encoded_matches(&self, other: &str) -> bool {
        let mut difference = (self.marker.len() ^ other.len()) as u8;
        for index in 0..CEF_MARKER_MAX_BYTES {
            let left = self.marker.as_bytes().get(index).copied().unwrap_or(0);
            let right = other.as_bytes().get(index).copied().unwrap_or(0);
            difference |= left ^ right;
        }
        difference == 0
    }

    #[cfg(test)]
    pub(super) fn decode_marker(&self) -> Result<CefLaunchMarker, super::CefMarkerError> {
        CefLaunchMarker::decode_unique(&[&self.marker])
    }
}

impl fmt::Debug for CefLaunchTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CefLaunchTicket([redacted])")
    }
}
