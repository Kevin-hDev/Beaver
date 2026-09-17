//! Resolve abbreviated IDs against an immutable snapshot of advertised tips.
use super::install_signal::InstallSignal;
use std::time::{Duration, Instant};

pub(super) fn resolve(
    repository: &git2::Repository,
    remote: &mut git2::Remote<'_>,
    tips: &[String],
    reference: &str,
    signal: &impl InstallSignal,
    remaining: &mut Duration,
) -> Result<String, git2::Error> {
    if tips.is_empty() || tips.len() > super::git_source::MAX_GIT_OBJECTS {
        return Err(git2::Error::from_str("remote references unavailable"));
    }
    // Never fetch moving ref names after the user waits: these exact tips and
    // their immutable ancestry define the resolution, even across a stopped fetch.
    let refs: Vec<&str> = tips.iter().map(String::as_str).collect();
    loop {
        if remaining.is_zero() {
            return Err(git2::Error::from_str("resolution expired"));
        }
        let started = Instant::now();
        let mut options =
            super::git_transport::fetch_options(started + *remaining, false, signal.clone())?;
        let result = remote.fetch(&refs, Some(&mut options), None);
        *remaining = remaining.saturating_sub(started.elapsed());
        let continued = signal
            .after_producer_stopped()
            .map_err(|_| git2::Error::from_str("resolution stopped"))?;
        if result.is_ok() {
            break;
        }
        if !continued {
            result?;
        }
    }
    repository
        .revparse_single(reference)?
        .peel_to_commit()
        .map(|commit| commit.id().to_string())
}
