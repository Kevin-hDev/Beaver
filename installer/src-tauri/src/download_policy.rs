use super::AttemptError;
use crate::error::InstallerError;
use crate::launch_args::PinnedRelease;

pub const MAX_RESPONSE_HEADERS: usize = 64;
pub const MAX_RESPONSE_HEADER_BYTES: usize = 64 * 1024;

pub(super) fn validate_headers(headers: &reqwest::header::HeaderMap) -> Result<(), AttemptError> {
    let bytes = headers
        .iter()
        .try_fold(0_usize, |total, (name, value)| {
            total.checked_add(name.as_str().len() + value.as_bytes().len())
        })
        .ok_or_else(|| AttemptError::download(false))?;
    if headers.len() > MAX_RESPONSE_HEADERS || bytes > MAX_RESPONSE_HEADER_BYTES {
        return Err(AttemptError::download(false));
    }
    Ok(())
}

pub(super) fn release_url(release: &PinnedRelease) -> Result<reqwest::Url, InstallerError> {
    let value = format!(
        "https://github.com/Kevin-hDev/Beaver/releases/download/v{}/{}",
        release.version, release.app_asset_name
    );
    (value.len() <= 4_096)
        .then(|| reqwest::Url::parse(&value).ok())
        .flatten()
        .ok_or(InstallerError::DownloadFailed)
}

pub fn redirect_allowed(url: &reqwest::Url) -> bool {
    url.as_str().len() <= 4_096
        && url.scheme() == "https"
        && url.host_str() == Some("release-assets.githubusercontent.com")
        && url.port_or_known_default() == Some(443)
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none()
}
