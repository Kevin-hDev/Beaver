use super::tool_pending_artifact_inspect::PreparedResult;
use super::tool_pending_artifact_errors::{artifact_error, file_error, invalid_result, too_large_result};
use super::types_tools::ToolResult;
use tokio_util::sync::CancellationToken;

pub(super) fn read_result(
    prepared: PreparedResult,
    cancel: &CancellationToken,
    key: Option<&[u8]>,
) -> ToolResult {
    read_result_with(prepared, cancel, key, |inspected, max_bytes, cancel| {
        crate::services::extensions::read_inspected_file_cancellable(
            inspected, max_bytes, cancel,
        )
    })
}

fn read_result_with<ReadFile>(
    mut prepared: PreparedResult,
    cancel: &CancellationToken,
    key: Option<&[u8]>,
    mut read_file: ReadFile,
) -> ToolResult
where
    ReadFile: FnMut(
        crate::services::extensions::InspectedFile,
        u64,
        &CancellationToken,
    ) -> Result<
        crate::services::extensions::VerifiedFile,
        crate::services::extensions::FileReadError,
    >,
{
    let mut artifacts = Vec::with_capacity(prepared.files.len());
    for (pending, inspected) in prepared.files {
        let Some(key) = key else {
            return invalid_result();
        };
        if cancel.is_cancelled() {
            return ToolResult::cancelled("Annulé.");
        }
        let verified = match read_file(
            inspected,
            crate::services::extensions::types::MAX_RESULT_BYTES as u64,
            cancel,
        ) {
            Ok(file) => file,
            Err(error) => return file_error(error),
        };
        let artifact = match crate::services::extensions::artifact_from_verified(
            verified,
            &pending.relative_path,
            pending.display_name.as_deref(),
            pending.purpose,
            key,
        ) {
            Ok(artifact) => artifact,
            Err(error) => return artifact_error(error),
        };
        artifacts.push(artifact);
    }
    if let Some((resource, inspected)) = prepared.resource {
        let verified = match read_file(
            inspected,
            crate::services::extensions::types::MAX_RESOURCE_FILE_BYTES as u64,
            cancel,
        ) {
            Ok(file) => file,
            Err(error) => return file_error(error),
        };
        let loaded = crate::services::extensions::LoadedResource {
            name: resource.name,
            extension_id: resource.extension_id.clone(),
            qualified_resource_id: resource.qualified_resource_id,
            catalog_fingerprint: resource.catalog_fingerprint,
            bytes: verified.bytes,
            signature: verified.signature,
        };
        if loaded.signature == crate::services::file_signature::FileSignature::Utf8 {
            if loaded.bytes.len() > crate::services::extensions::types::MAX_TEXT_RESOURCE_BYTES {
                return too_large_result();
            }
            let Ok(text) = String::from_utf8(loaded.bytes) else {
                return invalid_result();
            };
            prepared.result.content = format!("Resource source: {}\n\n{text}", resource.extension_id);
        } else if let Ok(Some(artifact)) = crate::services::extensions::extension_resource_artifact(loaded) {
            artifacts.push(artifact);
        } else {
            return invalid_result();
        }
    }
    for artifact in artifacts {
        prepared.result.push_ephemeral_artifact(artifact);
    }
    prepared.result
}

#[cfg(test)]
pub(super) fn read_result_cancelling_after_chunk(
    prepared: PreparedResult,
    cancel: &CancellationToken,
    key: Option<&[u8]>,
) -> ToolResult {
    let trigger = cancel.clone();
    read_result_with(prepared, cancel, key, move |inspected, max_bytes, cancel| {
        crate::services::extensions::read_inspected_file_cancellable_after_chunk(
            inspected,
            max_bytes,
            cancel,
            || trigger.cancel(),
        )
    })
}
