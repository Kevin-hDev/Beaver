use rand::RngCore;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use zeroize::Zeroizing;

use super::verify::{constant_time_token_eq, valid_health_token};
use super::WorkerError;

const HEALTH_TIMEOUT: Duration = Duration::from_secs(30);
const HEALTH_POLL: Duration = Duration::from_millis(100);

pub(crate) struct HealthToken {
    value: Zeroizing<String>,
    data_root: PathBuf,
}

impl HealthToken {
    pub(crate) fn generate(data_root: PathBuf) -> Result<Self, WorkerError> {
        let data_root = validate_data_root(&data_root)?;
        let mut bytes = Zeroizing::new([0_u8; 32]);
        rand::rngs::OsRng.fill_bytes(bytes.as_mut());
        let value = Zeroizing::new(hex::encode(bytes.as_ref()));
        if !valid_health_token(&value) {
            return Err(WorkerError);
        }
        let token = Self { value, data_root };
        if token.ack_path().exists() {
            return Err(WorkerError);
        }
        Ok(token)
    }

    pub(crate) fn value(&self) -> &str {
        self.value.as_str()
    }

    pub(crate) fn wait(&self) -> Result<(), WorkerError> {
        self.wait_for(HEALTH_TIMEOUT)
    }

    fn wait_for(&self, timeout: Duration) -> Result<(), WorkerError> {
        let started = Instant::now();
        loop {
            validate_health_directory(&self.data_root)?;
            let path = self.ack_path();
            if path.exists() {
                let result = read_ack(&path);
                let _ = std::fs::remove_file(path);
                return result;
            }
            if started.elapsed() >= timeout {
                return Err(WorkerError);
            }
            std::thread::sleep(HEALTH_POLL.min(timeout));
        }
    }

    fn ack_path(&self) -> PathBuf {
        self.data_root
            .join("update-health")
            .join(format!("{}.ok", self.value.as_str()))
    }
}

impl Drop for HealthToken {
    fn drop(&mut self) {
        if validate_health_directory(&self.data_root).is_ok() {
            let _ = std::fs::remove_file(self.ack_path());
        }
    }
}

fn read_ack(path: &Path) -> Result<(), WorkerError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| WorkerError)?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() || metadata.len() != 2 {
        return Err(WorkerError);
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let file = options.open(path).map_err(|_| WorkerError)?;
    let mut content = Vec::with_capacity(3);
    file.take(3)
        .read_to_end(&mut content)
        .map_err(|_| WorkerError)?;
    if content == b"ok" {
        Ok(())
    } else {
        Err(WorkerError)
    }
}

fn validate_data_root(data_root: &Path) -> Result<PathBuf, WorkerError> {
    let metadata = std::fs::symlink_metadata(data_root).map_err(|_| WorkerError)?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(WorkerError);
    }
    let root = std::fs::canonicalize(data_root).map_err(|_| WorkerError)?;
    validate_health_directory(&root)?;
    Ok(root)
}

fn validate_health_directory(data_root: &Path) -> Result<(), WorkerError> {
    let directory = data_root.join("update-health");
    let metadata = match std::fs::symlink_metadata(&directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(WorkerError),
    };
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(WorkerError);
    }
    let canonical = std::fs::canonicalize(directory).map_err(|_| WorkerError)?;
    if canonical.starts_with(data_root) {
        Ok(())
    } else {
        Err(WorkerError)
    }
}

pub(crate) fn token_in_arguments(arguments: &[std::ffi::OsString], expected: &str) -> bool {
    if arguments.len() > 64 {
        return false;
    }
    arguments.windows(2).any(|pair| {
        pair[0] == crate::services::update_health::UPDATE_HEALTH_ARG
            && pair[1]
                .to_str()
                .is_some_and(|actual| constant_time_token_eq(actual, expected))
    })
}

#[cfg(test)]
#[path = "health_tests.rs"]
mod tests;
