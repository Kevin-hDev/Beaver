#[tokio::test]
async fn resource_tool_refuses_missing_exact_identifier() {
    let result = super::tool_extension_resource::execute(&serde_json::json!({}), "session").await;

    assert_eq!(
        result.error.as_ref().map(|error| error.code.as_ref()),
        Some("resource_id_required")
    );
}

#[test]
fn text_resources_remain_textual_while_images_keep_ephemeral_artifacts() {
    let text = super::tool_extension_resource::resource_result(resource(
        "guide",
        b"guide".to_vec(),
        crate::services::file_signature::FileSignature::Utf8,
    ));
    assert!(text.content.contains("guide"));
    assert!(text.ephemeral_artifacts().is_empty());

    let image = super::tool_extension_resource::resource_result(resource(
        "preview",
        b"\x89PNG\r\n\x1a\n".to_vec(),
        crate::services::file_signature::FileSignature::Png,
    ));
    assert_eq!(image.ephemeral_artifacts().len(), 1);
    assert!(image.content.contains("artifact"));
    let serialized = serde_json::to_string(&image).expect("tool result serialization");
    assert!(!serialized.contains("sha256"));
    assert!(!serialized.contains("path"));
    assert!(!serialized.contains("grant"));
    assert!(!serialized.contains("iVBORw0KGgo="));
}

fn resource(
    local_id: &str,
    bytes: Vec<u8>,
    signature: crate::services::file_signature::FileSignature,
) -> crate::services::extensions::LoadedResource {
    crate::services::extensions::LoadedResource {
        name: local_id.into(),
        extension_id: "example".into(),
        qualified_resource_id: format!("extension:example:{local_id}"),
        catalog_fingerprint: "a".repeat(64),
        bytes,
        signature,
    }
}

#[test]
fn resource_failures_keep_the_contract_error_distinction() {
    use super::tool_result_contract::ToolErrorCategory;
    use crate::services::extensions::{error_codes, ResourceLoadError};

    for (error, code, category, retryable) in [
        (
            ResourceLoadError::InvalidId,
            error_codes::RESOURCE_INVALID,
            ToolErrorCategory::Validation,
            false,
        ),
        (
            ResourceLoadError::NotFound,
            error_codes::RESOURCE_NOT_FOUND,
            ToolErrorCategory::NotFound,
            false,
        ),
        (
            ResourceLoadError::TooLarge,
            error_codes::RESOURCE_TOO_LARGE,
            ToolErrorCategory::Validation,
            false,
        ),
        (
            ResourceLoadError::Unavailable,
            error_codes::RESOURCE_UNAVAILABLE,
            ToolErrorCategory::Unavailable,
            true,
        ),
    ] {
        let result = super::tool_extension_resource::failure(error);
        let error = result.error.expect("structured resource error");
        assert_eq!(error.code.as_ref(), code);
        assert_eq!(error.category, category);
        assert_eq!(error.retryable, retryable);
    }
}
