use super::constants::CEF_NONCE_BYTES;
use super::{CefLaunchMarker, CefMarkerError};
use std::fmt;
use zeroize::Zeroizing;

const NAME_LIMIT: usize = 128;
const OBJECT_TAGS: [&str; 4] = ["mailbox", "control", "admission", "closing"];

pub(in crate::services::browser) struct CefIpcNames {
    names: [Zeroizing<String>; 4],
}

impl CefIpcNames {
    pub(super) fn from_marker(marker: &CefLaunchMarker) -> Result<Self, CefMarkerError> {
        let mut nonce_hex = Zeroizing::new([0_u8; CEF_NONCE_BYTES * 2]);
        hex::encode_to_slice(marker.nonce_bytes(), nonce_hex.as_mut())
            .map_err(|_| CefMarkerError::Invalid)?;
        let nonce = std::str::from_utf8(nonce_hex.as_ref()).map_err(|_| CefMarkerError::Invalid)?;
        let names = OBJECT_TAGS.map(|tag| {
            Zeroizing::new(format!(
                "beaver-cef-{tag}-{}-{}-{nonce}",
                marker.slot(),
                marker.generation()
            ))
        });
        if names.iter().any(|name| name.len() > NAME_LIMIT) {
            return Err(CefMarkerError::Invalid);
        }
        Ok(Self { names })
    }

    #[cfg(test)]
    pub(super) fn are_pairwise_distinct(&self) -> bool {
        let mut duplicate = 0_u8;
        for left in 0..self.names.len() {
            for right in left + 1..self.names.len() {
                duplicate |= u8::from(
                    constant_time_text_difference(&self.names[left], &self.names[right]) == 0,
                );
            }
        }
        duplicate == 0
    }

    #[cfg(test)]
    pub(super) fn max_name_bytes(&self) -> usize {
        self.names.iter().map(|name| name.len()).max().unwrap_or(0)
    }

    #[cfg(test)]
    pub(super) fn constant_time_matches(&self, other: &Self) -> bool {
        let mut difference = 0_u8;
        for (left, right) in self.names.iter().zip(other.names.iter()) {
            difference |= constant_time_text_difference(left, right);
        }
        difference == 0
    }

    pub(super) fn get(&self, index: usize) -> Option<&str> {
        self.names.get(index).map(|name| name.as_str())
    }
}

impl fmt::Debug for CefIpcNames {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CefIpcNames([redacted])")
    }
}

#[cfg(test)]
fn constant_time_text_difference(left: &str, right: &str) -> u8 {
    let mut difference = (left.len() ^ right.len()) as u8;
    for index in 0..NAME_LIMIT {
        let left_byte = left.as_bytes().get(index).copied().unwrap_or(0);
        let right_byte = right.as_bytes().get(index).copied().unwrap_or(0);
        difference |= left_byte ^ right_byte;
    }
    difference
}
