use crate::models::agent_session_contract::{AgentSessionView, ToolArtifactRecordView};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use super::tool_artifact_record::{ToolArtifactRecord, ToolArtifactSource, ToolArtifactStatus};
use super::types_session::AgentSession;

// History has its own I/O ceiling: never scale hashing work with session length.
const MAX_HISTORY_VERIFICATION_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Default)]
pub(super) struct HistoryVerificationBudget {
    reserved_bytes: u64,
}

impl HistoryVerificationBudget {
    fn admit(&mut self, maximum: u64) -> bool {
        // Reserve the reader's worst case, including its one-byte overflow probe.
        // Persisted sizes cannot understate the cost of reading a modified file.
        let Some(next) = self.reserved_bytes.checked_add(maximum).and_then(|n| n.checked_add(1)) else {
            return false;
        };
        if next > MAX_HISTORY_VERIFICATION_BYTES { return false; }
        self.reserved_bytes = next;
        true
    }
}

pub(crate) async fn apply(session: &AgentSession, view: &mut AgentSessionView) {
    let records = records(session);
    let view_records = view_records_mut(view);
    if records.len() != view_records.len() {
        log::warn!("session_artifact_view_mismatch");
        return;
    }
    let needs_workspace_key = records
        .iter()
        .any(|record| matches!(record.source, ToolArtifactSource::WorkspaceFile { .. }));
    let key = needs_workspace_key
        .then(crate::services::attachment_access::attachment_key)
        .transpose()
        .ok()
        .flatten();
    let mut budget = HistoryVerificationBudget::default();
    for (record, view_record) in records.into_iter().zip(view_records) {
        view_record.verification = match &record.source {
            ToolArtifactSource::WorkspaceFile { .. } => {
                verify_workspace_bounded(record, key.as_deref().map(Vec::as_slice), &mut budget).await
            }
            ToolArtifactSource::ExtensionResource { .. } => {
                if budget.admit(crate::services::extensions::types::MAX_RESOURCE_FILE_BYTES as u64) {
                    Some(verify_extension(record).await)
                } else {
                    None
                }
            }
        };
    }
}

pub(super) async fn verify_workspace_bounded(
    record: &ToolArtifactRecord,
    key: Option<&[u8]>,
    budget: &mut HistoryVerificationBudget,
) -> Option<ToolArtifactStatus> {
    if !budget.admit(crate::services::extensions::types::MAX_RESULT_BYTES as u64) {
        return None;
    }
    Some(verify_workspace(record, key).await)
}

pub(super) async fn verify_workspace(
    record: &ToolArtifactRecord,
    key: Option<&[u8]>,
) -> ToolArtifactStatus {
    let ToolArtifactSource::WorkspaceFile { path, grant } = &record.source else {
        unreachable!()
    };
    let Some(key) = key else {
        return ToolArtifactStatus::Inaccessible;
    };
    let expected_bytes = record.bytes;
    let expected_sha256 = record.sha256.clone();
    let key = Zeroizing::new(key.to_vec());
    let path = path.clone();
    let grant = Zeroizing::new(grant.clone());
    tokio::task::spawn_blocking(move || {
        match std::path::Path::new(&path).try_exists() {
            Ok(false) => return ToolArtifactStatus::Absent,
            Err(_) => return ToolArtifactStatus::Inaccessible,
            Ok(true) => {}
        }
        match crate::services::attachment_access::read_verified(
            &path,
            &grant,
            &key,
            crate::services::extensions::types::MAX_RESULT_BYTES as u64,
        ) {
            Ok(file) if artifact_matches(expected_bytes, &expected_sha256, &file.bytes) => {
                ToolArtifactStatus::Intact
            }
            Ok(_) => ToolArtifactStatus::Modified,
            Err(_) if matches!(std::path::Path::new(&path).try_exists(), Ok(false)) => {
                ToolArtifactStatus::Absent
            }
            Err(_) => ToolArtifactStatus::Inaccessible,
        }
    })
    .await
    .unwrap_or(ToolArtifactStatus::Inaccessible)
}

async fn verify_extension(record: &ToolArtifactRecord) -> ToolArtifactStatus {
    let ToolArtifactSource::ExtensionResource { resource_id, .. } = &record.source else {
        unreachable!()
    };
    resource_status_from_load(
        record,
        crate::services::extensions::load_extension_resource_for_history(resource_id).await,
    )
}

pub(super) fn resource_status_from_load(
    record: &ToolArtifactRecord,
    loaded: Result<
        crate::services::extensions::LoadedResource,
        crate::services::extensions::ResourceLoadError,
    >,
) -> ToolArtifactStatus {
    let ToolArtifactSource::ExtensionResource {
        catalog_fingerprint,
        ..
    } = &record.source
    else {
        unreachable!()
    };
    match loaded {
        Ok(resource) if resource.catalog_fingerprint != *catalog_fingerprint => {
            ToolArtifactStatus::Modified
        }
        Ok(resource) if artifact_matches(record.bytes, &record.sha256, &resource.bytes) => {
            ToolArtifactStatus::Intact
        }
        Ok(_) => ToolArtifactStatus::Modified,
        Err(crate::services::extensions::ResourceLoadError::NotFound) => {
            ToolArtifactStatus::Absent
        }
        Err(_) => ToolArtifactStatus::Inaccessible,
    }
}

fn artifact_matches(expected_bytes: u64, expected_sha256: &str, bytes: &[u8]) -> bool {
    bytes.len() as u64 == expected_bytes
        && hex::encode(Sha256::digest(bytes)).eq_ignore_ascii_case(expected_sha256)
}

fn records(session: &AgentSession) -> Vec<&ToolArtifactRecord> {
    session
        .messages
        .iter()
        .flat_map(|message| {
            message
                .tool_activities
                .iter()
                .flatten()
                .flat_map(|tool| tool.artifacts.iter())
                .chain(message.segments.iter().flatten().flat_map(|segment| {
                    segment
                        .tools
                        .iter()
                        .flat_map(|tool| tool.artifacts.iter())
                }))
        })
        .collect()
}

fn view_records_mut(view: &mut AgentSessionView) -> Vec<&mut ToolArtifactRecordView> {
    view.messages
        .iter_mut()
        .flat_map(|message| {
            message
                .tool_activities
                .iter_mut()
                .flatten()
                .flat_map(|tool| tool.artifacts.iter_mut())
                .chain(message.segments.iter_mut().flatten().flat_map(|segment| {
                    segment
                        .tools
                        .iter_mut()
                        .flat_map(|tool| tool.artifacts.iter_mut())
                }))
        })
        .collect()
}
