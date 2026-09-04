use serde_json::Value;

use super::types_tools::ToolResult;

pub(crate) const NAME: &str = "load_extension_resource";

pub(crate) async fn execute(args: &Value, session_id: &str) -> ToolResult {
    let Some(resource_id) = args.get("resource_id").and_then(Value::as_str) else {
        return ToolResult::validation("resource_id_required", "Identifiant de ressource requis.");
    };
    match crate::services::extensions::load_extension_resource_for_session(resource_id, session_id)
        .await
    {
        Ok(resource) if resource.signature == crate::services::file_signature::FileSignature::Utf8 => {
            let Ok(text) = String::from_utf8(resource.bytes) else {
                return ToolResult::unavailable(
                    crate::services::extensions::error_codes::RESOURCE_UNAVAILABLE,
                    "Ressource d'extension indisponible.",
                    true,
                );
            };
            ToolResult::ok(format!("Resource source: {}\n\n{text}", resource.extension_id))
                .with_display_summary(resource.name)
        }
        Ok(resource) => ToolResult::ok(format!(
            "Resource source: {}\nType: {}\nContent is not loaded in this version.",
            resource.extension_id,
            resource.signature.mime()
        ))
        .with_display_summary(resource.name),
        Err(error) => failure(error),
    }
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
