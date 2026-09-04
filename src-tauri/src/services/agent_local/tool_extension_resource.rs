use serde_json::Value;

use super::types_tools::ToolResult;

pub(crate) const NAME: &str = "load_extension_resource";

pub(crate) async fn execute(args: &Value, session_id: &str) -> ToolResult {
    let Some(resource_id) = args.get("resource_id").and_then(Value::as_str) else {
        return ToolResult::validation("resource_id_required", "Identifiant de ressource requis.");
    };
    match crate::services::extensions::prepare_extension_resource_for_session(resource_id, session_id).await {
        Ok(resource) => pending_resource_result(resource),
        Err(error) => failure(error),
    }
}

fn pending_resource_result(resource: crate::services::extensions::PreparedResource) -> ToolResult {
    let mut result = ToolResult::ok(format!("Resource source: {}", resource.extension_id))
        .with_display_summary(resource.name.clone());
    if result
        .set_pending_extension_resource(resource.into())
        .is_err()
    {
        return unavailable();
    }
    result
}

#[cfg(test)]
pub(super) fn resource_result(
    resource: crate::services::extensions::LoadedResource,
) -> ToolResult {
    if resource.signature == crate::services::file_signature::FileSignature::Utf8 {
        let summary = resource.name.clone();
        let source = resource.extension_id.clone();
        let Ok(text) = String::from_utf8(resource.bytes) else {
            return unavailable();
        };
        return ToolResult::ok(format!("Resource source: {source}\n\n{text}"))
            .with_display_summary(summary);
    }
    let source = resource.extension_id.clone();
    let mime = resource.signature.mime();
    match crate::services::extensions::extension_resource_artifact(resource) {
        Ok(Some(artifact)) => ToolResult::ok(format!(
            "Resource source: {source}\nType: {mime}\nContent is available as an artifact."
        ))
        .with_display_summary(artifact.metadata.name.clone())
        .with_ephemeral_artifact(artifact),
        _ => unavailable(),
    }
}

fn unavailable() -> ToolResult {
    ToolResult::unavailable(
        crate::services::extensions::error_codes::RESOURCE_UNAVAILABLE,
        "Ressource d'extension indisponible.",
        true,
    )
}

pub(super) fn failure(error: crate::services::extensions::ResourceLoadError) -> ToolResult {
    use crate::services::extensions::{error_codes, ResourceLoadError};

    match error {
        ResourceLoadError::InvalidId => ToolResult::validation(
            error_codes::RESOURCE_INVALID,
            "Ressource d'extension invalide.",
        ),
        ResourceLoadError::NotFound => ToolResult::not_found(
            error_codes::RESOURCE_NOT_FOUND,
            "Ressource d'extension introuvable.",
        ),
        ResourceLoadError::TooLarge => ToolResult::validation(
            error_codes::RESOURCE_TOO_LARGE,
            "Ressource d'extension trop volumineuse.",
        ),
        ResourceLoadError::Unavailable => ToolResult::unavailable(
            error_codes::RESOURCE_UNAVAILABLE,
            "Ressource d'extension indisponible.",
            true,
        ),
    }
}
