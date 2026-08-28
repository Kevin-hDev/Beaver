use std::io::Read;
use std::path::PathBuf;

pub(crate) struct VerifiedAttachment {
    pub canonical_path: PathBuf,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerifiedAttachmentError {
    Access,
    Limit,
    Unavailable,
}

pub(crate) fn read_verified(
    raw: &str,
    access_grant: &str,
    key: &[u8],
    max_bytes: u64,
) -> Result<VerifiedAttachment, VerifiedAttachmentError> {
    read_verified_after(raw, access_grant, key, max_bytes, || {})
}

pub(super) fn read_verified_after<F>(
    raw: &str,
    access_grant: &str,
    key: &[u8],
    max_bytes: u64,
    after_grant: F,
) -> Result<VerifiedAttachment, VerifiedAttachmentError>
where
    F: FnOnce(),
{
    if max_bytes > super::attachment_access::MAX_ATTACHMENT_SIZE {
        return Err(VerifiedAttachmentError::Limit);
    }
    let (canonical_path, expected_identity, original_file) =
        super::attachment_access::verify_access_grant_with_identity(raw, access_grant, key)
            .map_err(|_| VerifiedAttachmentError::Access)?;
    // Keep this handle open so the OS cannot recycle its identity during the path check.
    after_grant();
    let mut file = super::private_store::open_regular_single_link(&canonical_path)
        .map_err(|_| VerifiedAttachmentError::Access)?
        .ok_or(VerifiedAttachmentError::Access)?;
    let metadata = file
        .metadata()
        .map_err(|_| VerifiedAttachmentError::Unavailable)?;
    let opened_identity = super::attachment_access_identity::from_file(&file)
        .ok_or(VerifiedAttachmentError::Access)?;
    if !metadata.is_file() || opened_identity != expected_identity {
        return Err(VerifiedAttachmentError::Access);
    }
    drop(original_file);
    if metadata.len() > max_bytes {
        return Err(VerifiedAttachmentError::Limit);
    }
    let read_limit = max_bytes
        .checked_add(1)
        .ok_or(VerifiedAttachmentError::Limit)?;
    let capacity = usize::try_from(metadata.len()).map_err(|_| VerifiedAttachmentError::Limit)?;
    let mut bytes = Vec::with_capacity(capacity);
    Read::by_ref(&mut file)
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|_| VerifiedAttachmentError::Unavailable)?;
    if bytes.len() as u64 > max_bytes {
        return Err(VerifiedAttachmentError::Limit);
    }
    Ok(VerifiedAttachment {
        canonical_path,
        bytes,
    })
}
