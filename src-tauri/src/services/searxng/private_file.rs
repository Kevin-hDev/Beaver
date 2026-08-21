use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub(super) fn read_bounded(path: &Path, max_bytes: u64) -> Result<Vec<u8>, ()> {
    match crate::services::private_store::read_bounded_regular(path, max_bytes).map_err(|_| ())? {
        crate::services::private_store::BoundedFile::Content(bytes) if !bytes.is_empty() => {
            Ok(bytes)
        }
        _ => Err(()),
    }
}

pub(super) fn create_private(path: &Path) -> Result<File, ()> {
    let parent = path.parent().ok_or(())?;
    let metadata = std::fs::symlink_metadata(parent).map_err(|_| ())?;
    if !metadata.file_type().is_dir() {
        return Err(());
    }
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => {
            let existing = open_read(path)?;
            if !crate::services::private_store::file_is_single_link_regular(&existing) {
                return Err(());
            }
            std::fs::remove_file(path).map_err(|_| ())?;
        }
        Ok(_) => return Err(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return Err(()),
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    configure_no_follow(&mut options);
    let file = options.open(path).map_err(|_| ())?;
    if !crate::services::private_store::file_is_single_link_regular(&file) {
        return Err(());
    }
    Ok(file)
}

pub(super) fn read_tail(path: &Path, max_bytes: u64) -> Result<Vec<u8>, ()> {
    let mut file = open_read(path)?;
    if !crate::services::private_store::file_is_single_link_regular(&file) {
        return Err(());
    }
    let metadata = file.metadata().map_err(|_| ())?;
    let start = metadata.len().saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start)).map_err(|_| ())?;
    let mut bytes = Vec::with_capacity((metadata.len() - start) as usize);
    file.take(max_bytes)
        .read_to_end(&mut bytes)
        .map_err(|_| ())?;
    Ok(bytes)
}

fn open_read(path: &Path) -> Result<File, ()> {
    let mut options = OpenOptions::new();
    options.read(true);
    configure_no_follow(&mut options);
    options.open(path).map_err(|_| ())
}

#[cfg(unix)]
fn configure_no_follow(options: &mut OpenOptions) {
    use std::os::unix::fs::OpenOptionsExt;

    options
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .mode(0o600);
}

#[cfg(windows)]
fn configure_no_follow(options: &mut OpenOptions) {
    use std::os::windows::fs::OpenOptionsExt;

    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_writer_refuses_a_hard_link() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("target");
        let link = root.path().join("log");
        std::fs::write(&target, b"keep").unwrap();
        std::fs::hard_link(&target, &link).unwrap();

        assert!(create_private(&link).is_err());
        assert_eq!(std::fs::read(target).unwrap(), b"keep");
    }

    #[cfg(unix)]
    #[test]
    fn bounded_reader_never_follows_a_symlink() {
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("target");
        let link = root.path().join("manifest");
        std::fs::write(&target, b"secret").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        assert!(read_bounded(&link, 512).is_err());
    }
}
