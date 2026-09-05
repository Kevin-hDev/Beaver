use super::tool_pending_artifact_errors::invalid_result;
use super::types_tools::ToolResult;

pub(super) async fn revalidate_resources(results: &mut [Option<ToolResult>]) {
    for result in results.iter_mut().filter_map(Option::as_mut) {
        let Some(resource) = result.pending_extension_resource() else {
            continue;
        };
        let valid = crate::services::extensions::revalidate_extension_resource_for_resolution(
            &resource.session_id,
            &resource.extension_id,
            &resource.qualified_resource_id,
            &resource.catalog_fingerprint,
            &resource.root,
            &resource.relative_path,
        )
        .await
        .is_ok();
        if !valid {
            *result = invalid_result();
        }
    }
}
