use std::path::Path;
use std::time::Duration;

const MAX_SCAN_DEPTH: usize = 16;
const MAX_SCAN_ENTRIES: usize = 65_536;
const STALE_TEMP_AGE: Duration = Duration::from_secs(60 * 60);

pub(super) fn purge_stale_atomic_temps_logged(root: &Path) {
    let started = std::time::Instant::now();
    let result = purge_stale_atomic_temps(root);
    ::log::info!(
        "[private-store] operation=temp-cleanup elapsed_ms={}",
        started.elapsed().as_millis()
    );
    if result.is_err() {
        // Hygiene must never make an otherwise valid profile impossible to open.
        ::log::warn!("[private-store] operation=temp-cleanup result=incomplete");
    }
}

pub(super) fn purge_stale_atomic_temps(root: &Path) -> Result<(), String> {
    let mut stack = vec![(root.to_path_buf(), 0_usize)];
    let mut scanned = 0_usize;
    while let Some((directory, depth)) = stack.pop() {
        for entry in std::fs::read_dir(directory).map_err(|_| super::private_store_error())? {
            let entry = entry.map_err(|_| super::private_store_error())?;
            scanned = scanned
                .checked_add(1)
                .ok_or_else(super::private_store_error)?;
            if scanned > MAX_SCAN_ENTRIES {
                return Err(super::private_store_error());
            }
            let file_type = entry
                .file_type()
                .map_err(|_| super::private_store_error())?;
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                if depth >= MAX_SCAN_DEPTH {
                    // Deep user content is valid; the bounded hygiene pass simply does not enter it.
                    continue;
                }
                stack.push((entry.path(), depth + 1));
            } else if file_type.is_file()
                && is_owned_atomic_temp(&entry.file_name())
                && is_stale(&entry.path())
            {
                std::fs::remove_file(entry.path()).map_err(|_| super::private_store_error())?;
            }
        }
    }
    Ok(())
}

fn is_stale(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .and_then(|metadata| metadata.modified())
        .and_then(|modified| modified.elapsed().map_err(std::io::Error::other))
        .map(|elapsed| elapsed >= STALE_TEMP_AGE)
        .unwrap_or(false)
}

fn is_owned_atomic_temp(name: &std::ffi::OsStr) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };
    let body = name
        .strip_prefix('.')
        .and_then(|value| value.strip_suffix(".tmp"));
    let Some((target, nonce)) = body.and_then(|value| value.rsplit_once('.')) else {
        return false;
    };
    // This exact CSPRNG-backed namespace is reserved by atomic_write.
    !target.is_empty()
        && target.len() <= 255
        && nonce.len() == 32
        && nonce.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::purge_stale_atomic_temps;

    fn make_stale(path: &std::path::Path) {
        let old = std::time::SystemTime::now()
            - super::STALE_TEMP_AGE
            - std::time::Duration::from_secs(1);
        let times = std::fs::FileTimes::new().set_modified(old);
        std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .unwrap()
            .set_times(times)
            .unwrap();
    }

    #[test]
    fn removes_only_regular_owned_atomic_temporaries() {
        let root = tempfile::tempdir().unwrap();
        let stale = root
            .path()
            .join(".config.json.0123456789abcdef0123456789abcdef.tmp");
        let unrelated = root.path().join("config.json.tmp");
        std::fs::write(root.path().join("config.json"), b"current").unwrap();
        std::fs::write(&stale, b"stale").unwrap();
        std::fs::write(&unrelated, b"keep").unwrap();
        make_stale(&stale);

        purge_stale_atomic_temps(root.path()).unwrap();

        assert!(!stale.exists());
        assert!(unrelated.exists());
    }

    #[test]
    fn preserves_a_recent_temporary_from_another_live_writer() {
        let root = tempfile::tempdir().unwrap();
        let active = root
            .path()
            .join(".config.json.0123456789abcdef0123456789abcdef.tmp");
        std::fs::write(root.path().join("config.json"), b"current").unwrap();
        std::fs::write(&active, b"in progress").unwrap();

        purge_stale_atomic_temps(root.path()).unwrap();

        assert!(active.exists());
    }

    #[test]
    fn removes_a_stale_first_write_residue_without_a_destination() {
        let root = tempfile::tempdir().unwrap();
        let shaped = root
            .path()
            .join(".config.json.0123456789abcdef0123456789abcdef.tmp");
        std::fs::write(&shaped, b"interrupted first write").unwrap();
        make_stale(&shaped);

        purge_stale_atomic_temps(root.path()).unwrap();

        assert!(!shaped.exists());
    }

    #[cfg(unix)]
    #[test]
    fn never_follows_a_symbolic_link() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let stale = outside
            .path()
            .join(".config.json.0123456789abcdef0123456789abcdef.tmp");
        std::fs::write(&stale, b"outside").unwrap();
        symlink(outside.path(), root.path().join("linked")).unwrap();

        purge_stale_atomic_temps(root.path()).unwrap();

        assert!(stale.exists());
    }

    #[test]
    fn deep_user_content_does_not_block_startup_cleanup() {
        let root = tempfile::tempdir().unwrap();
        let mut directory = root.path().to_path_buf();
        for _ in 0..=super::MAX_SCAN_DEPTH {
            directory.push("nested");
            std::fs::create_dir(&directory).unwrap();
        }

        purge_stale_atomic_temps(root.path()).unwrap();
        assert!(directory.exists());
    }
}
