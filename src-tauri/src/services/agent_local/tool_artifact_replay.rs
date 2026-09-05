use std::sync::Arc;

use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use super::tool_artifact_record::{self, ToolArtifactPurpose, ToolArtifactRecord, ToolArtifactSource, ToolArtifactStatus};

const ABSENT_NOTE: &str = "Saved extension output is no longer available.";
const MODIFIED_NOTE: &str = "Saved extension output changed and cannot be replayed.";
const INACCESSIBLE_NOTE: &str = "Saved extension output cannot be accessed.";
pub(super) const UNSUPPORTED_PREVIEW_NOTE: &str =
    "Saved extension preview is not a supported image.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReplayedArtifact {
    pub name: String,
    pub mime_type: String,
    pub purpose: ToolArtifactPurpose,
    pub bytes: Arc<[u8]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArtifactReplay {
    pub status: ToolArtifactStatus,
    pub artifact: Option<ReplayedArtifact>,
    pub note: Option<&'static str>,
}

pub(crate) async fn replay(
    record: &ToolArtifactRecord,
    workspace_key: Option<&[u8]>,
) -> ArtifactReplay {
    if tool_artifact_record::validate(std::slice::from_ref(record)).is_err() {
        return unavailable(ToolArtifactStatus::Inaccessible);
    }
    match &record.source {
        ToolArtifactSource::WorkspaceFile { path, grant } => {
            replay_workspace(record, path, grant, workspace_key).await
        }
        ToolArtifactSource::ExtensionResource { resource_id, .. } => {
            replay_extension(record, resource_id).await
        }
    }
}

async fn replay_workspace(
    record: &ToolArtifactRecord,
    path: &str,
    grant: &str,
    workspace_key: Option<&[u8]>,
) -> ArtifactReplay {
    let Some(key) = workspace_key else {
        return unavailable(ToolArtifactStatus::Inaccessible);
    };
    let record = record.clone();
    let path = path.to_owned();
    let grant = Zeroizing::new(grant.to_owned());
    let key = Zeroizing::new(key.to_vec());
    tokio::task::spawn_blocking(move || {
        match std::path::Path::new(&path).try_exists() {
            Ok(false) => return unavailable(ToolArtifactStatus::Absent),
            Err(_) => return unavailable(ToolArtifactStatus::Inaccessible),
            Ok(true) => {}
        }
        match crate::services::attachment_access::read_verified(
            &path,
            &grant,
            &key,
            crate::services::extensions::types::MAX_RESULT_BYTES as u64,
        ) {
            Ok(file) => replay_bytes(&record, file.bytes),
            Err(_) if matches!(std::path::Path::new(&path).try_exists(), Ok(false)) => {
                unavailable(ToolArtifactStatus::Absent)
            }
            Err(_) => unavailable(ToolArtifactStatus::Inaccessible),
        }
    })
    .await
    .unwrap_or_else(|_| unavailable(ToolArtifactStatus::Inaccessible))
}

async fn replay_extension(record: &ToolArtifactRecord, resource_id: &str) -> ArtifactReplay {
    replay_extension_from_load(
        record,
        crate::services::extensions::load_extension_resource_for_history(resource_id).await,
    )
}

fn replay_extension_from_load(
    record: &ToolArtifactRecord,
    loaded: Result<crate::services::extensions::LoadedResource, crate::services::extensions::ResourceLoadError>,
) -> ArtifactReplay {
    let ToolArtifactSource::ExtensionResource {
        resource_id,
        catalog_fingerprint,
    } = &record.source
    else {
        return unavailable(ToolArtifactStatus::Inaccessible);
    };
    match loaded {
        Ok(resource)
            if resource.qualified_resource_id != *resource_id
                || resource.catalog_fingerprint != *catalog_fingerprint =>
        {
            unavailable(ToolArtifactStatus::Modified)
        }
        Ok(resource) => replay_bytes(record, resource.bytes),
        Err(crate::services::extensions::ResourceLoadError::NotFound) => {
            unavailable(ToolArtifactStatus::Absent)
        }
        Err(_) => unavailable(ToolArtifactStatus::Inaccessible),
    }
}

fn replay_bytes(record: &ToolArtifactRecord, bytes: Vec<u8>) -> ArtifactReplay {
    let matches = bytes.len() as u64 == record.bytes
        && hex::encode(Sha256::digest(&bytes)) == record.sha256;
    if !matches {
        return unavailable(ToolArtifactStatus::Modified);
    }
    ArtifactReplay {
        status: ToolArtifactStatus::Intact,
        artifact: Some(ReplayedArtifact {
            name: record.name.clone(),
            mime_type: record.mime_type.clone(),
            purpose: record.purpose.clone(),
            bytes: Arc::from(bytes),
        }),
        note: None,
    }
}

fn unavailable(status: ToolArtifactStatus) -> ArtifactReplay {
    let note = match status {
        ToolArtifactStatus::Absent => ABSENT_NOTE,
        ToolArtifactStatus::Modified => MODIFIED_NOTE,
        ToolArtifactStatus::Inaccessible => INACCESSIBLE_NOTE,
        ToolArtifactStatus::Intact => unreachable!(),
    };
    ArtifactReplay {
        status,
        artifact: None,
        note: Some(note),
    }
}

#[cfg(test)]
#[path = "tool_artifact_replay_tests.rs"]
mod tests;
