use super::*;

pub(crate) async fn download_with_retries(
    client: reqwest::Client,
    url: reqwest::Url,
    release: &PinnedRelease,
    run: &OwnedTempRun,
) -> Result<PathBuf, InstallerError> {
    download_attempts(client, url, release, run, &CancellationToken::new(), |_| {}).await
}

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
