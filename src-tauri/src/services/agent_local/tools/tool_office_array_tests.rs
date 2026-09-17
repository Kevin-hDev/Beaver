use super::tool_document_write::write_document;
use super::tool_image_process::transform_image;
use super::tool_office_limits::{
    MAX_DOCUMENT_BLOCKS, MAX_IMAGE_OPERATIONS, MAX_SPREADSHEET_OPERATIONS,
};
use super::tool_spreadsheet_write::write_spreadsheet;
use super::types_tools::ToolResult;
use serde_json::{json, Value};

fn error_code(result: &ToolResult) -> Option<&str> {
    result.error.as_ref().map(|error| error.code.as_ref())
}

#[tokio::test]
async fn spreadsheet_operation_limit_has_a_specific_error() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("limited.xlsx");
    let operations = Value::Array(vec![json!({}); MAX_SPREADSHEET_OPERATIONS + 1]);

    let result = write_spreadsheet(path.to_str().unwrap(), &operations, directory.path()).await;

    assert_eq!(
        error_code(&result),
        Some("spreadsheet_operation_limit_exceeded")
    );
    assert!(!path.exists());
}

#[tokio::test]
async fn document_block_limit_has_a_specific_error() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("limited.docx");
    let blocks = Value::Array(vec![json!({}); MAX_DOCUMENT_BLOCKS + 1]);

    let result = write_document(path.to_str().unwrap(), &blocks, directory.path()).await;

    assert_eq!(error_code(&result), Some("document_block_limit_exceeded"));
    assert!(!path.exists());
}

#[tokio::test]
async fn image_operation_limit_has_a_specific_error() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.png");
    let output = directory.path().join("output.png");
    image::RgbImage::new(1, 1).save(&input).unwrap();
    let operations = Value::Array(vec![json!({}); MAX_IMAGE_OPERATIONS + 1]);

    let result = transform_image(
        input.to_str().unwrap(),
        output.to_str().unwrap(),
        &operations,
        directory.path(),
    )
    .await;

    assert_eq!(error_code(&result), Some("image_operation_limit_exceeded"));
    assert!(!output.exists());
}
