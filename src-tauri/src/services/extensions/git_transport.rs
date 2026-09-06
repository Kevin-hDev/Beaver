use super::{git_source::MAX_GIT_OBJECTS, install_signal::InstallSignal};
use git2::{AutotagOption, FetchOptions, RemoteCallbacks};
use std::time::Instant;

pub(super) fn fetch_options(
    deadline: Instant,
    shallow: bool,
    cancellation: impl InstallSignal,
) -> Result<FetchOptions<'static>, git2::Error> {
    let mut fetch = FetchOptions::new();
    if shallow {
        fetch.depth(1);
    }
    fetch
        .download_tags(AutotagOption::Auto)
        .remote_callbacks(callbacks(deadline, cancellation)?);
    Ok(fetch)
}

pub(super) fn callbacks(
    deadline: Instant,
    cancellation: impl InstallSignal,
) -> Result<RemoteCallbacks<'static>, git2::Error> {
    let config = git2::Config::open_default()
        .map_err(|_| git2::Error::from_str("git configuration unavailable"))?;
    let mut credentials =
        crate::services::git::remote_credentials::CredentialProvider::new(config, None);
    let mut callbacks = RemoteCallbacks::new();
    let credential_cancellation = cancellation.clone();
    callbacks.credentials(move |url, username, allowed| {
        if credential_cancellation.producer_should_stop() || Instant::now() >= deadline {
            return Err(git2::Error::new(
                git2::ErrorCode::Timeout,
                git2::ErrorClass::Net,
                "extension clone expired",
            ));
        }
        credentials.credentials(url, username, allowed)
    });
    let transfer_cancellation = cancellation.clone();
    callbacks.transfer_progress(move |progress| {
        transfer_cancellation.downloaded(progress.received_bytes() as u64)
            && Instant::now() < deadline
            && progress.total_objects() <= MAX_GIT_OBJECTS
    });
    Ok(callbacks)
}
