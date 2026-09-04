use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileReadError {
    Access,
    Limit,
    NotFound,
    Unavailable,
}

pub(crate) struct VerifiedFile {
    pub bytes: Vec<u8>,
    pub signature: crate::services::file_signature::FileSignature,
}

pub(crate) fn read(
    root: &Path,
    relative: &str,
    max_bytes: u64,
) -> Result<VerifiedFile, FileReadError> {
    read_with_hooks(root, relative, max_bytes, || {}, || {})
}

#[cfg(test)]
pub(super) fn read_after<F>(
    root: &Path,
    relative: &str,
    max_bytes: u64,
    after_initial_open: F,
) -> Result<VerifiedFile, FileReadError>
where
    F: FnOnce(),
{
    read_with_hooks(root, relative, max_bytes, after_initial_open, || {})
}

#[cfg(test)]
pub(super) fn read_after_content<F>(
    root: &Path,
    relative: &str,
    max_bytes: u64,
    after_content_read: F,
) -> Result<VerifiedFile, FileReadError>
where
    F: FnOnce(),
{
    read_with_hooks(root, relative, max_bytes, || {}, after_content_read)
}

fn read_with_hooks<F, G>(
    root: &Path,
    relative: &str,
    max_bytes: u64,
    after_initial_open: F,
    after_content_read: G,
) -> Result<VerifiedFile, FileReadError>
where
    F: FnOnce(),
    G: FnOnce(),
{
    let path = confined_path(root, relative)?;
    let (_, expected_identity) = open_with_identity(&path)?;
    after_initial_open();
    let (mut file, opened_identity) = open_with_identity(&path)?;
    if opened_identity != expected_identity {
        return Err(FileReadError::Access);
    }
    let metadata = file.metadata().map_err(|_| FileReadError::Unavailable)?;
    if metadata.len() > max_bytes {
        return Err(FileReadError::Limit);
    }
    let read_limit = max_bytes.checked_add(1).ok_or(FileReadError::Limit)?;
    let capacity = usize::try_from(metadata.len()).map_err(|_| FileReadError::Limit)?;
    let mut bytes = Vec::with_capacity(capacity);
    Read::by_ref(&mut file)
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|_| FileReadError::Unavailable)?;
    if bytes.len() as u64 > max_bytes {
        return Err(FileReadError::Limit);
    }
    after_content_read();
    let (_, current_identity) = open_with_identity(&path)?;
    if crate::services::attachment_access_identity::from_file(&file) != Some(opened_identity)
        || current_identity != opened_identity
        || path.canonicalize().map_err(|_| FileReadError::Access)? != path
    {
        return Err(FileReadError::Access);
    }
    Ok(VerifiedFile {
        signature: crate::services::file_signature::classify(&bytes),
        bytes,
    })
}

fn confined_path(root: &Path, relative: &str) -> Result<PathBuf, FileReadError> {
    super::contribution_path::validate(relative).map_err(|_| FileReadError::Access)?;
    let root = root.canonicalize().map_err(|_| FileReadError::Access)?;
    if !root.is_dir() {
        return Err(FileReadError::Access);
    }
    let mut unchecked = root.clone();
    for component in relative.split('/') {
        unchecked.push(component);
        let metadata = std::fs::symlink_metadata(&unchecked).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                FileReadError::NotFound
            } else {
                FileReadError::Access
            }
        })?;
        if metadata.file_type().is_symlink() {
            return Err(FileReadError::Access);
        }
    }
    let path = root.join(relative).canonicalize().map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            FileReadError::NotFound
        } else {
            FileReadError::Access
        }
    })?;
    (path.starts_with(&root) && path != root)
        .then_some(path)
        .ok_or(FileReadError::Access)
}

fn open_with_identity(
    path: &Path,
) -> Result<
    (
        std::fs::File,
        crate::services::attachment_access_identity::FileIdentity,
    ),
    FileReadError,
> {
    let file = crate::services::private_store::open_regular_single_link(path)
        .map_err(|_| FileReadError::Access)?
        .ok_or(FileReadError::NotFound)?;
    let identity = crate::services::attachment_access_identity::from_file(&file)
        .ok_or(FileReadError::Access)?;
    Ok((file, identity))
}
