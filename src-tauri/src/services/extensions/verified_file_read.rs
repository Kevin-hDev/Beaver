use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileReadError {
    Access,
    Limit,
    NotFound,
    Unavailable,
    Cancelled,
}

pub(crate) struct InspectedFile {
    file: std::fs::File,
    pub size: u64,
    pub canonical_path: PathBuf,
    pub identity: crate::services::attachment_access_identity::FileIdentity,
}

pub(crate) struct VerifiedFile {
    pub bytes: Vec<u8>,
    pub signature: crate::services::file_signature::FileSignature,
    pub canonical_path: PathBuf,
    pub identity: crate::services::attachment_access_identity::FileIdentity,
}

pub(crate) fn inspect(
    root: &Path,
    relative: &str,
    max_bytes: u64,
) -> Result<InspectedFile, FileReadError> {
    let path = confined_path(root, relative)?;
    let (file, identity) = open_with_identity(&path)?;
    let metadata = file.metadata().map_err(|_| FileReadError::Unavailable)?;
    if metadata.len() > max_bytes {
        return Err(FileReadError::Limit);
    }
    Ok(InspectedFile {
        file,
        size: metadata.len(),
        canonical_path: path,
        identity,
    })
}

pub(crate) fn read_inspected(
    inspected: InspectedFile,
    max_bytes: u64,
) -> Result<VerifiedFile, FileReadError> {
    read_inspected_with_hook(inspected, max_bytes, None, || {}, || {})
}

pub(crate) fn read_inspected_cancellable(
    inspected: InspectedFile,
    max_bytes: u64,
    cancel: &tokio_util::sync::CancellationToken,
) -> Result<VerifiedFile, FileReadError> {
    read_inspected_with_hook(inspected, max_bytes, Some(cancel), || {}, || {})
}

#[cfg(test)]
pub(crate) fn read_inspected_cancellable_after_chunk<G>(
    inspected: InspectedFile,
    max_bytes: u64,
    cancel: &tokio_util::sync::CancellationToken,
    after_chunk: G,
) -> Result<VerifiedFile, FileReadError>
where
    G: FnMut(),
{
    read_inspected_with_hook(inspected, max_bytes, Some(cancel), || {}, after_chunk)
}

pub(crate) fn read(
    root: &Path,
    relative: &str,
    max_bytes: u64,
) -> Result<VerifiedFile, FileReadError> {
    read_inspected(inspect(root, relative, max_bytes)?, max_bytes)
}

pub(super) fn read_inspected_with_hook<F, G>(
    mut inspected: InspectedFile,
    max_bytes: u64,
    cancel: Option<&tokio_util::sync::CancellationToken>,
    after_content_read: F,
    mut after_chunk: G,
) -> Result<VerifiedFile, FileReadError>
where
    F: FnOnce(),
    G: FnMut(),
{
    let metadata = inspected
        .file
        .metadata()
        .map_err(|_| FileReadError::Unavailable)?;
    if metadata.len() != inspected.size {
        return Err(FileReadError::Access);
    }
    if metadata.len() > max_bytes {
        return Err(FileReadError::Limit);
    }
    let read_limit = max_bytes.checked_add(1).ok_or(FileReadError::Limit)?;
    let capacity = usize::try_from(inspected.size).map_err(|_| FileReadError::Limit)?;
    let mut bytes = Vec::with_capacity(capacity);
    let mut remaining = read_limit;
    let mut buffer = [0_u8; 64 * 1024];
    while remaining > 0 {
        if cancel.is_some_and(tokio_util::sync::CancellationToken::is_cancelled) {
            return Err(FileReadError::Cancelled);
        }
        let length = usize::try_from(remaining.min(buffer.len() as u64))
            .map_err(|_| FileReadError::Limit)?;
        let count = inspected
            .file
            .read(&mut buffer[..length])
            .map_err(|_| FileReadError::Unavailable)?;
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..count]);
        remaining -= u64::try_from(count).map_err(|_| FileReadError::Limit)?;
        after_chunk();
    }
    if bytes.len() as u64 > max_bytes {
        return Err(FileReadError::Limit);
    }
    after_content_read();
    if cancel.is_some_and(tokio_util::sync::CancellationToken::is_cancelled) {
        return Err(FileReadError::Cancelled);
    }
    let (_, current_identity) = open_with_identity(&inspected.canonical_path)?;
    if crate::services::attachment_access_identity::from_file(&inspected.file)
        != Some(inspected.identity)
        || inspected
            .file
            .metadata()
            .map_err(|_| FileReadError::Unavailable)?
            .len()
            != inspected.size
        || current_identity != inspected.identity
        || inspected
            .canonical_path
            .canonicalize()
            .map_err(|_| FileReadError::Access)?
            != inspected.canonical_path
    {
        return Err(FileReadError::Access);
    }
    if cancel.is_some_and(tokio_util::sync::CancellationToken::is_cancelled) {
        return Err(FileReadError::Cancelled);
    }
    Ok(VerifiedFile {
        signature: crate::services::file_signature::classify(&bytes),
        bytes,
        canonical_path: inspected.canonical_path,
        identity: inspected.identity,
    })
}

fn confined_path(root: &Path, relative: &str) -> Result<PathBuf, FileReadError> {
    super::contribution_path::validate(relative).map_err(|_| FileReadError::Access)?;
    let root = root.canonicalize().map_err(|_| FileReadError::Access)?;
    if !root.is_dir() {
        return Err(FileReadError::Access);
    }
    let mut unchecked = root.clone();
    let mut components = relative.split('/').peekable();
    while let Some(component) = components.next() {
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
        let valid_kind = if components.peek().is_some() {
            metadata.is_dir()
        } else {
            metadata.is_file()
        };
        if !valid_kind {
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
