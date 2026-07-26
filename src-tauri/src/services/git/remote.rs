use git2::{ErrorClass, ErrorCode, PushOptions, RemoteCallbacks};
use serde::Serialize;
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use zeroize::Zeroizing;

use super::{remote_credentials, remote_status, remote_target, repo};
use crate::services::brand::DISPLAY_NAME;

pub use remote_status::{status, RemoteStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PushError {
    #[error("remote unavailable")]
    NoRemote,
    #[error("authentication required")]
    AuthenticationRequired,
    #[error("remote permission denied")]
    PermissionDenied,
    #[error("remote branch changed")]
    RemoteChanged,
    #[error("network unavailable")]
    NetworkUnavailable,
    #[error("git context changed")]
    ContextChanged,
    #[error("internal git error")]
    InternalError,
}

pub fn remote_requires_github_token(repo_path: &Path) -> bool {
    let Ok(repository) = repo::open(repo_path) else {
        return false;
    };
    let Ok(branch) = remote_target::current_branch(&repository) else {
        return false;
    };
    let Ok(name) = remote_target::branch_name(&branch) else {
        return false;
    };
    remote_target::selected_remote_url(&repository, &name)
        .is_some_and(|url| url.starts_with("https://github.com/"))
}

pub fn push_current(
    repo_path: &Path,
    expected_branch: Option<&str>,
    github_token: Option<Zeroizing<String>>,
) -> Result<(), PushError> {
    let repository = repo::open(repo_path).map_err(|_| PushError::InternalError)?;
    let mut branch =
        remote_target::current_branch(&repository).map_err(|_| PushError::InternalError)?;
    let name = remote_target::branch_name(&branch).map_err(|_| PushError::InternalError)?;
    if expected_branch.is_some_and(|expected| expected != name) {
        return Err(PushError::ContextChanged);
    }
    let remote_name =
        remote_target::choose_remote(&repository, &name).ok_or(PushError::NoRemote)?;
    let mut remote = repository
        .find_remote(&remote_name)
        .map_err(|_| PushError::NoRemote)?;
    let config = git2::Config::open_default().map_err(|_| PushError::InternalError)?;
    let mut credentials = remote_credentials::CredentialProvider::new(config, github_token);
    let rejection = Arc::new(Mutex::new(None));
    let rejection_callback = Arc::clone(&rejection);

    let mut callbacks = RemoteCallbacks::new();
    callbacks
        .credentials(move |url, username, allowed| credentials.credentials(url, username, allowed));
    callbacks.push_update_reference(move |_, status| {
        if let Some(status) = status {
            let mut result = rejection_callback
                .lock()
                .map_err(|_| git2::Error::from_str("push callback unavailable"))?;
            *result = Some(classify_rejection(status));
        }
        Ok(())
    });
    let mut options = PushOptions::new();
    options.remote_callbacks(callbacks);
    let refspec = format!("refs/heads/{name}:refs/heads/{name}");
    remote
        .push(&[refspec.as_str()], Some(&mut options))
        .map_err(map_push_error)?;
    if let Some(error) = rejection
        .lock()
        .map_err(|_| PushError::InternalError)?
        .take()
    {
        return Err(error);
    }

    track_pushed_branch(&repository, &mut branch, &remote_name, &name)
}

fn track_pushed_branch(
    repository: &git2::Repository,
    branch: &mut git2::Branch<'_>,
    remote: &str,
    name: &str,
) -> Result<(), PushError> {
    let oid = branch.get().target().ok_or(PushError::InternalError)?;
    repository
        .reference(
            &format!("refs/remotes/{remote}/{name}"),
            oid,
            true,
            &format!("track branch pushed by {DISPLAY_NAME}"),
        )
        .map_err(|_| PushError::InternalError)?;
    branch
        .set_upstream(Some(&format!("{remote}/{name}")))
        .map_err(|_| PushError::InternalError)
}

pub(super) fn map_push_error(error: git2::Error) -> PushError {
    if error.code() == ErrorCode::NotFastForward {
        return PushError::RemoteChanged;
    }
    if error.code() == ErrorCode::Auth {
        return PushError::AuthenticationRequired;
    }
    if matches!(error.code(), ErrorCode::Certificate | ErrorCode::Timeout)
        || matches!(error.class(), ErrorClass::Net | ErrorClass::Ssl)
    {
        return PushError::NetworkUnavailable;
    }
    classify_message(error.message())
}

fn classify_rejection(message: &str) -> PushError {
    let error = classify_message(message);
    if error == PushError::InternalError {
        PushError::PermissionDenied
    } else {
        error
    }
}

fn classify_message(message: &str) -> PushError {
    let message = message.to_ascii_lowercase();
    if message.contains("non-fast-forward") || message.contains("fetch first") {
        PushError::RemoteChanged
    } else if message.contains("auth") || message.contains("credential") {
        PushError::AuthenticationRequired
    } else if message.contains("403")
        || message.contains("permission")
        || message.contains("denied")
        || message.contains("protected branch")
        || message.contains("gh013")
    {
        PushError::PermissionDenied
    } else if message.contains("resolve host")
        || message.contains("connection")
        || message.contains("network")
        || message.contains("timed out")
    {
        PushError::NetworkUnavailable
    } else {
        PushError::InternalError
    }
}
