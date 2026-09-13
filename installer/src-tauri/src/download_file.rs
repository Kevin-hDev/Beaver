use super::{AttemptError, DownloadProgress};
use crate::launch_args::PinnedRelease;
use crate::temp_ownership::OwnedTempRun;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::Duration;
use subtle::ConstantTimeEq;
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

pub(super) async fn write_verified(
    response: reqwest::Response,
    release: &PinnedRelease,
    run: &OwnedTempRun,
    cancellation: &CancellationToken,
    progress: impl Fn(DownloadProgress),
    idle_timeout: Duration,
) -> Result<PathBuf, AttemptError> {
    let part = run.path().join(format!("{}.part", release.app_asset_name));
    let final_path = run.path().join(&release.app_asset_name);
    let mut cleanup = PartialFile::create(&part)?;
    let mut file = tokio::fs::File::from_std(cleanup.file.take().expect("partial file"));
    let mut stream = response.bytes_stream();
    let mut downloaded = 0_u64;
    let mut hasher = Sha256::new();
    loop {
        let next = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(AttemptError::download(false)),
            next = tokio::time::timeout(idle_timeout, stream.next()) => next,
        }
        .map_err(|_| AttemptError::download(true))?;
        let Some(chunk) = next else { break };
        let chunk = chunk
            .map_err(|error| AttemptError::download(error.is_connect() || error.is_timeout()))?;
        downloaded = downloaded
            .checked_add(chunk.len() as u64)
            .filter(|size| *size <= release.app_asset_size)
            .ok_or_else(AttemptError::integrity)?;
        hasher.update(&chunk);
        file.write_all(&chunk)
            .await
            .map_err(|_| AttemptError::download(false))?;
        progress(DownloadProgress {
            completed: downloaded,
            total: release.app_asset_size,
        });
    }
    let actual: [u8; 32] = hasher.finalize().into();
    let expected = decode_sha256(&release.app_asset_sha256).ok_or_else(AttemptError::integrity)?;
    if downloaded != release.app_asset_size || !bool::from(actual.ct_eq(&expected)) {
        return Err(AttemptError::integrity());
    }
    file.flush()
        .await
        .map_err(|_| AttemptError::download(false))?;
    file.sync_all()
        .await
        .map_err(|_| AttemptError::download(false))?;
    drop(file);
    if fs::symlink_metadata(&final_path).is_ok() {
        return Err(AttemptError::integrity());
    }
    fs::rename(&part, &final_path).map_err(|_| AttemptError::download(false))?;
    cleanup.disarm();
    sync_directory(run.path()).map_err(|_| AttemptError::download(false))?;
    Ok(final_path)
}

fn decode_sha256(value: &str) -> Option<[u8; 32]> {
    let mut decoded = [0_u8; 32];
    hex::decode_to_slice(value, &mut decoded).ok()?;
    Some(decoded)
}

struct PartialFile {
    path: PathBuf,
    file: Option<File>,
    armed: bool,
}

impl PartialFile {
    fn create(path: &Path) -> Result<Self, AttemptError> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options
            .open(path)
            .map_err(|_| AttemptError::download(false))?;
        Ok(Self {
            path: path.to_path_buf(),
            file: Some(file),
            armed: true,
        })
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for PartialFile {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> std::io::Result<()> {
    File::open(path)?.sync_all()
}

#[cfg(windows)]
fn sync_directory(path: &Path) -> std::io::Result<()> {
    use std::os::windows::fs::OpenOptionsExt;
    OpenOptions::new()
        .read(true)
        .custom_flags(0x0200_0000)
        .open(path)?
        .sync_all()
}
