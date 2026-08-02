use super::tool_image_process::transform_image;
use super::tool_result_contract::ToolResultStatus;
use super::types_tools::ToolResult;

fn source(directory: &std::path::Path) -> std::path::PathBuf {
    let path = directory.join("input.png");
    image::RgbImage::new(4, 4).save(&path).unwrap();
    path
}

fn error_code(result: &ToolResult) -> Option<&str> {
    result.error.as_ref().map(|error| error.code.as_ref())
}

#[tokio::test]
async fn unsupported_output_format_is_a_validation_error() {
    let directory = tempfile::tempdir().unwrap();
    let input = source(directory.path());
    let output = directory.path().join("output.txt");

    let result = transform_image(
        input.to_str().unwrap(),
        output.to_str().unwrap(),
        &serde_json::Value::Null,
        directory.path(),
    )
    .await;

    assert_eq!(error_code(&result), Some("image_output_format_unsupported"));
    assert!(!output.exists());
}

#[tokio::test]
async fn invalid_resize_mode_is_not_silently_treated_as_fit() {
    let directory = tempfile::tempdir().unwrap();
    let input = source(directory.path());
    let output = directory.path().join("output.png");
    let operations = serde_json::json!([
        {"type": "resize", "width": 2, "height": 2, "mode": "unknown"}
    ]);

    let result = transform_image(
        input.to_str().unwrap(),
        output.to_str().unwrap(),
        &operations,
        directory.path(),
    )
    .await;

    assert_eq!(error_code(&result), Some("image_resize_mode_invalid"));
    assert!(!output.exists());
}

#[tokio::test]
async fn missing_quality_value_has_a_specific_error() {
    let directory = tempfile::tempdir().unwrap();
    let input = source(directory.path());
    let output = directory.path().join("output.jpg");
    let operations = serde_json::json!([{"type": "quality"}]);

    let result = transform_image(
        input.to_str().unwrap(),
        output.to_str().unwrap(),
        &operations,
        directory.path(),
    )
    .await;

    assert_eq!(error_code(&result), Some("image_quality_required"));
    assert!(!output.exists());
}

#[tokio::test]
async fn ignored_quality_is_reported_as_a_partial_result() {
    let directory = tempfile::tempdir().unwrap();
    let input = source(directory.path());
    let output = directory.path().join("output.png");
    let operations = serde_json::json!([{"type": "quality", "value": 80}]);

    let result = transform_image(
        input.to_str().unwrap(),
        output.to_str().unwrap(),
        &operations,
        directory.path(),
    )
    .await;

    assert_eq!(result.status, ToolResultStatus::Partial);
    assert!(result.warnings.iter().any(|warning| warning.contains("ignorée")));
    assert!(output.exists());
}
