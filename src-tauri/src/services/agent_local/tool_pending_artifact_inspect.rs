use super::tool_artifact::{PendingArtifact, PendingExtensionResource};
use super::tool_pending_artifact_errors::{file_error, too_large_result};
use super::types_tools::ToolResult;
use tokio_util::sync::CancellationToken;

pub(super) struct PreparedResult {
    pub(super) result: ToolResult,
    pub(super) files: Vec<(PendingArtifact, crate::services::extensions::InspectedFile)>,
    pub(super) resource: Option<(
        PendingExtensionResource,
        crate::services::extensions::InspectedFile,
    )>,
    pub(super) bytes: u64,
}

pub(super) fn inspect_result(
    mut result: ToolResult,
    working_dir: &std::path::Path,
    cancel: &CancellationToken,
) -> Result<PreparedResult, ToolResult> {
    let pending = result.take_pending_artifacts();
    let pending_resource = result.take_pending_extension_resource();
    let mut bytes = 0_u64;
    let mut files = Vec::with_capacity(pending.len());
    for artifact in pending {
        if cancel.is_cancelled() {
            return Err(ToolResult::cancelled("Annulé."));
        }
        let inspected = crate::services::extensions::inspect_verified_file(
            working_dir,
            &artifact.relative_path,
            crate::services::extensions::types::MAX_RESULT_BYTES as u64,
        )
        .map_err(file_error)?;
        bytes = add_result_bytes(bytes, inspected.size)?;
        files.push((artifact, inspected));
    }
    let resource = inspect_resource(pending_resource, cancel, &mut bytes)?;
    Ok(PreparedResult {
        result,
        files,
        resource,
        bytes,
    })
}

fn inspect_resource(
    resource: Option<PendingExtensionResource>,
    cancel: &CancellationToken,
    bytes: &mut u64,
) -> Result<
    Option<(
        PendingExtensionResource,
        crate::services::extensions::InspectedFile,
    )>,
    ToolResult,
> {
    let Some(resource) = resource else {
        return Ok(None);
    };
    if cancel.is_cancelled() {
        return Err(ToolResult::cancelled("Annulé."));
    }
    let inspected = crate::services::extensions::inspect_verified_file(
        &resource.root,
        &resource.relative_path,
        crate::services::extensions::types::MAX_RESOURCE_FILE_BYTES as u64,
    )
    .map_err(file_error)?;
    *bytes = add_result_bytes(*bytes, inspected.size)?;
    Ok(Some((resource, inspected)))
}

fn add_result_bytes(current: u64, next: u64) -> Result<u64, ToolResult> {
    current
        .checked_add(next)
        .filter(|bytes| *bytes <= crate::services::extensions::types::MAX_RESULT_BYTES as u64)
        .ok_or_else(too_large_result)
}
