use super::durable_fs::{OllamaFsError, OllamaFsErrorKind, OllamaFsOperation};
use super::error::OllamaErrorCode;

#[test]
fn durable_storage_diagnostic_keeps_kind_operation_and_os_code() {
    let error = OllamaFsError::from_os_code(OllamaFsErrorKind::PermissionDenied, 5)
        .at(OllamaFsOperation::SyncParent);

    assert_eq!(
        super::storage_error::diagnostic_fields(error),
        (
            OllamaFsErrorKind::PermissionDenied,
            Some(OllamaFsOperation::SyncParent),
            Some(5),
        )
    );
    assert_eq!(
        super::storage_error::durable("cleanup-remove-tree", error),
        OllamaErrorCode::OllamaStorageUnavailable
    );
}

#[test]
fn cleanup_routes_durable_failures_through_the_diagnostic_authority() {
    let source = include_str!("cleanup.rs");
    assert!(!source.contains("map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)"));
    assert!(source.matches("storage_error::durable").count() >= 5);
}
