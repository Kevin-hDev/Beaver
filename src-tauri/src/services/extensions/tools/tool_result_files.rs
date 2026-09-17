use std::path::Path;

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileResultError {
    #[cfg(test)]
    Access,
    Limit,
    #[cfg(test)]
    NotFound,
    Unavailable,
    Grant,
}

#[cfg(test)]
pub(crate) fn read_workspace_file(
    root: &Path,
    relative: &str,
    display_name: Option<&str>,
    purpose: crate::services::agent_local::tool_artifact::ArtifactPurpose,
    key: &[u8],
) -> Result<crate::services::agent_local::tool_artifact::EphemeralArtifact, FileResultError> {
    if display_name.is_some_and(|name| {
        name.is_empty()
            || name.chars().count() > super::types::MAX_EXTENSION_NAME_CHARS
            || name.chars().any(char::is_control)
    }) {
        return Err(FileResultError::Access);
    }
    let loaded =
        super::verified_file_read::read(root, relative, super::types::MAX_RESULT_BYTES as u64)
            .map_err(classify_read_error)?;
    artifact_from_verified(loaded, relative, display_name, purpose, key)
}

pub(crate) fn artifact_from_verified(
    loaded: super::verified_file_read::VerifiedFile,
    relative: &str,
    display_name: Option<&str>,
    purpose: crate::services::agent_local::tool_artifact::ArtifactPurpose,
    key: &[u8],
) -> Result<crate::services::agent_local::tool_artifact::EphemeralArtifact, FileResultError> {
    let grant = crate::services::attachment_access::grant_verified_path(
        &loaded.canonical_path,
        loaded.identity,
        key,
    )
    .map_err(|_| FileResultError::Grant)?;
    let name = display_name
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .or_else(|| {
            Path::new(relative)
                .file_name()
                .and_then(|name| name.to_str())
                .map(bounded_visible_name)
        })
        .ok_or(FileResultError::Unavailable)?;
    let sha256 = hex::encode(Sha256::digest(&loaded.bytes));
    let bytes = u64::try_from(loaded.bytes.len()).map_err(|_| FileResultError::Limit)?;
    let metadata = crate::services::agent_local::tool_artifact::ArtifactMetadata {
        name,
        mime_type: loaded.signature.mime().to_string(),
        bytes,
        sha256,
        purpose,
        source: crate::services::agent_local::tool_artifact::ArtifactSource::WorkspaceFile {
            path: loaded.canonical_path,
            grant,
        },
    };
    Ok(
        crate::services::agent_local::tool_artifact::EphemeralArtifact {
            metadata,
            bytes: loaded.bytes,
        },
    )
}

fn bounded_visible_name(value: &str) -> String {
    value
        .chars()
        .take(super::types::MAX_EXTENSION_NAME_CHARS)
        .collect()
}

#[cfg(test)]
pub(crate) fn classify_read_error(
    error: super::verified_file_read::FileReadError,
) -> FileResultError {
    match error {
        super::verified_file_read::FileReadError::Access => FileResultError::Access,
        super::verified_file_read::FileReadError::Limit => FileResultError::Limit,
        super::verified_file_read::FileReadError::NotFound => FileResultError::NotFound,
        super::verified_file_read::FileReadError::Unavailable => FileResultError::Unavailable,
        super::verified_file_read::FileReadError::Cancelled => FileResultError::Unavailable,
    }
}
