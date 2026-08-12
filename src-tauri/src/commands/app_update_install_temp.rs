use std::fs::{self, File, OpenOptions};
use std::path::PathBuf;

use uuid::Uuid;

const MAX_TEMP_ATTEMPTS: usize = 8;

#[derive(Debug)]
pub(crate) struct TemporaryUpdate {
    path: PathBuf,
    remove_on_drop: bool,
}

impl TemporaryUpdate {
    pub(crate) fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub(crate) fn persist(mut self) -> PathBuf {
        self.remove_on_drop = false;
        self.path.clone()
    }
}

impl Drop for TemporaryUpdate {
    fn drop(&mut self) {
        if self.remove_on_drop {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub(crate) fn create_unique_temp_file(
    prefix: &str,
    suffix: &str,
) -> Result<(TemporaryUpdate, File), String> {
    for _ in 0..MAX_TEMP_ATTEMPTS {
        let path = std::env::temp_dir().join(format!("{prefix}-{}{suffix}", Uuid::new_v4()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&path) {
            Ok(file) => {
                return Ok((
                    TemporaryUpdate {
                        path,
                        remove_on_drop: true,
                    },
                    file,
                ));
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => {
                ::log::error!("[update] create temp file: {e}");
                return Err("update-write-error".to_string());
            }
        }
    }
    Err("update-write-error".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporary_update_is_deleted_unless_persisted() {
        let (temporary, file) = create_unique_temp_file("beaver-delete-test", ".tmp").unwrap();
        let path = temporary.path().to_path_buf();
        drop(file);
        drop(temporary);
        assert!(!path.exists());

        let (temporary, file) = create_unique_temp_file("beaver-keep-test", ".tmp").unwrap();
        let path = temporary.persist();
        drop(file);
        assert!(path.exists());
        fs::remove_file(path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn temporary_update_uses_private_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let (temporary, file) =
            create_unique_temp_file("beaver-private-update-test", ".tmp").unwrap();
        let path = temporary.path();
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        drop(file);
        drop(temporary);
    }
}
