#[tokio::test]
async fn resource_tool_refuses_missing_exact_identifier() {
    let result = super::tool_extension_resource::execute(&serde_json::json!({}), "session").await;

    assert_eq!(
        result.error.as_ref().map(|error| error.code.as_ref()),
        Some("resource_id_required")
    );
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
