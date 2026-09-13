use crate::error::InstallerError;
use crate::launch_args::PinnedRelease;
use crate::temp_ownership::OwnedTempRun;
use reqwest::header::{ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_LENGTH, LOCATION};
use std::path::PathBuf;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[path = "download_file.rs"]
mod file;
#[path = "download_policy.rs"]
mod policy;

pub use policy::{redirect_allowed, MAX_RESPONSE_HEADERS, MAX_RESPONSE_HEADER_BYTES};

pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
pub const IDLE_CHUNK_TIMEOUT: Duration = Duration::from_secs(30);
pub const ASSET_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const MAX_REDIRECTS: usize = 3;
const MAX_ATTEMPTS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadProgress {
    pub completed: u64,
    pub total: u64,
}

pub fn percent(progress: DownloadProgress) -> Option<u8> {
    (progress.total > 0).then(|| {
        progress
            .completed
            .saturating_mul(100)
            .checked_div(progress.total)
            .unwrap_or(0)
            .min(100) as u8
    })
}

pub async fn download_pinned_asset(
    release: &PinnedRelease,
    run: &OwnedTempRun,
    cancellation: &CancellationToken,
    progress: impl Fn(DownloadProgress) + Copy,
) -> Result<PathBuf, InstallerError> {
    let client = reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(ASSET_TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| InstallerError::DownloadFailed)?;
    let url = policy::release_url(release)?;
    tokio::time::timeout(
        ASSET_TIMEOUT,
        download_attempts(client, url, release, run, cancellation, progress),
    )
    .await
    .unwrap_or(Err(InstallerError::DownloadFailed))
}

async fn download_attempts(
    client: reqwest::Client,
    url: reqwest::Url,
    release: &PinnedRelease,
    run: &OwnedTempRun,
    cancellation: &CancellationToken,
    progress: impl Fn(DownloadProgress) + Copy,
) -> Result<PathBuf, InstallerError> {
    if cancellation.is_cancelled() {
        return Err(InstallerError::DownloadFailed);
    }
    match file::reuse_verified(release, run, cancellation).await {
        Ok(Some(path)) => {
            progress(DownloadProgress {
                completed: release.app_asset_size,
                total: release.app_asset_size,
            });
            return Ok(path);
        }
        Ok(None) => {}
        Err(InstallerError::IntegrityFailed) => file::discard_invalid(release, run)?,
        Err(error) => return Err(error),
    }
    for attempt in 0..MAX_ATTEMPTS {
        match attempt_download(
            client.clone(),
            url.clone(),
            release,
            run,
            cancellation,
            progress,
            IDLE_CHUNK_TIMEOUT,
        )
        .await
        {
            Ok(path) => return Ok(path),
            Err(error) if error.retryable && attempt + 1 < MAX_ATTEMPTS => {
                tokio::select! {
                    biased;
                    _ = cancellation.cancelled() => return Err(InstallerError::DownloadFailed),
                    _ = tokio::time::sleep(Duration::from_secs((attempt + 1) as u64)) => {}
                }
            }
            Err(error) => return Err(error.public),
        }
    }
    Err(InstallerError::DownloadFailed)
}

#[cfg(test)]
pub(crate) async fn download_with_retries(
    client: reqwest::Client,
    url: reqwest::Url,
    release: &PinnedRelease,
    run: &OwnedTempRun,
) -> Result<PathBuf, InstallerError> {
    download_attempts(client, url, release, run, &CancellationToken::new(), |_| {}).await
}

#[cfg(test)]
pub(crate) async fn download_once(
    client: reqwest::Client,
    url: reqwest::Url,
    release: &PinnedRelease,
    run: &OwnedTempRun,
    cancellation: &CancellationToken,
    progress: impl Fn(DownloadProgress),
) -> Result<PathBuf, InstallerError> {
    attempt_download(
        client,
        url,
        release,
        run,
        cancellation,
        progress,
        IDLE_CHUNK_TIMEOUT,
    )
    .await
    .map_err(|error| error.public)
}

#[cfg(test)]
pub(crate) async fn download_once_with_timeouts(
    client: reqwest::Client,
    url: reqwest::Url,
    release: &PinnedRelease,
    run: &OwnedTempRun,
    idle_timeout: Duration,
    total_timeout: Duration,
) -> Result<PathBuf, InstallerError> {
    tokio::time::timeout(
        total_timeout,
        attempt_download(
            client,
            url,
            release,
            run,
            &CancellationToken::new(),
            |_| {},
            idle_timeout,
        ),
    )
    .await
    .map_err(|_| InstallerError::DownloadFailed)?
    .map_err(|error| error.public)
}

pub(super) struct AttemptError {
    pub(super) public: InstallerError,
    retryable: bool,
}

impl AttemptError {
    pub(super) fn download(retryable: bool) -> Self {
        Self {
            public: InstallerError::DownloadFailed,
            retryable,
        }
    }

    pub(super) fn integrity() -> Self {
        Self {
            public: InstallerError::IntegrityFailed,
            retryable: false,
        }
    }
}

async fn attempt_download(
    client: reqwest::Client,
    mut url: reqwest::Url,
    release: &PinnedRelease,
    run: &OwnedTempRun,
    cancellation: &CancellationToken,
    progress: impl Fn(DownloadProgress),
    idle_timeout: Duration,
) -> Result<PathBuf, AttemptError> {
    let mut redirects = 0;
    let response = loop {
        let sent = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(AttemptError::download(false)),
            sent = client.get(url.clone()).header(ACCEPT_ENCODING, "identity").send() => sent,
        }
        .map_err(|error| AttemptError::download(error.is_connect() || error.is_timeout()))?;
        policy::validate_headers(sent.headers())?;
        if sent.status().is_redirection() {
            if redirects == MAX_REDIRECTS {
                return Err(AttemptError::download(false));
            }
            let locations: Vec<_> = sent.headers().get_all(LOCATION).iter().collect();
            if locations.len() != 1 {
                return Err(AttemptError::download(false));
            }
            let value = locations[0]
                .to_str()
                .map_err(|_| AttemptError::download(false))?;
            url = url.join(value).map_err(|_| AttemptError::download(false))?;
            if !policy::redirect_allowed(&url) {
                return Err(AttemptError::download(false));
            }
            redirects += 1;
            continue;
        }
        if sent.status().as_u16() != 200 {
            return Err(AttemptError::download(sent.status().is_server_error()));
        }
        break sent;
    };

    if response
        .headers()
        .get(CONTENT_LENGTH)
        .map(|value| {
            value
                .to_str()
                .ok()
                .and_then(|text| text.parse::<u64>().ok())
        })
        .is_some_and(|length| length != Some(release.app_asset_size))
    {
        return Err(AttemptError::integrity());
    }
    if response
        .headers()
        .get(CONTENT_ENCODING)
        .is_some_and(|value| !value.as_bytes().eq_ignore_ascii_case(b"identity"))
    {
        return Err(AttemptError::download(false));
    }
    file::write_verified(response, release, run, cancellation, progress, idle_timeout).await
}
