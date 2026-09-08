use super::{session_types::MAX_BROWSER_TABS, url_policy::validate_browser_url};
use std::time::Duration;

// Global, like ViewRecency: sessions share ten native views, not ten each.
pub(super) const MAX_FAVICONS: usize = MAX_BROWSER_TABS;
pub(super) const MAX_PENDING_SNAPSHOTS: usize = MAX_FAVICONS * 2;
pub(super) const MAX_CANDIDATES: usize = 8;
pub(super) const MAX_DOWNLOADS: usize = 4;
pub(super) const MAX_PNG_BYTES: usize = 32 * 1024;
pub(super) const MAX_PIXELS: i32 = 64;
#[cfg(native_browser)]
pub(super) const REQUEST_DIP: u32 = 32;
pub(super) const DOWNLOAD_DEADLINE: Duration = Duration::from_secs(5);
pub(super) const MAX_REVISION: u64 = (1 << 53) - 1;

pub(super) fn candidates(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut result = Vec::new();
    for raw in values.into_iter().take(MAX_CANDIDATES) {
        if let Ok(url) = validate_browser_url(&raw) {
            if !result.iter().any(|value| value == url.as_str()) {
                result.push(url.as_str().to_owned());
            }
        }
    }
    result
}

pub(super) fn valid_dimensions(width: i32, height: i32) -> bool {
    (1..=MAX_PIXELS).contains(&width) && (1..=MAX_PIXELS).contains(&height)
}

#[cfg(any(test, native_browser))]
// Diagnostics only: DownloadImage exposes no cancellation handle.
pub(super) const STALL_DIAGNOSTIC_DELAY: Duration = Duration::from_secs(60);
