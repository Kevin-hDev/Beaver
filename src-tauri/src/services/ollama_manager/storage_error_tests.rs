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
    let cleanup = include_str!("cleanup.rs");
    let temporaries = include_str!("temporary_recovery.rs");
    assert!(!cleanup.contains("map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)"));
    assert!(!temporaries.contains("map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)"));
    for operation in [
        "cleanup-rename",
        "migration-marker-write",
        "cleanup-remove-tree",
    ] {
        assert!(
            cleanup.contains(operation),
            "missing diagnostic {operation}"
        );
    }
    assert!(temporaries.contains("storage_error::durable(context, error)"));
}

#[test]
fn reviewed_j3_storage_boundaries_use_the_diagnostic_authority() {
    for (name, source) in [
        ("adoption", include_str!("adoption.rs")),
        ("bundle receipt", include_str!("bundle_receipt.rs")),
        ("install archives", include_str!("install_archives.rs")),
        ("cleanup inspection", include_str!("cleanup_inspection.rs")),
        (
            "update preflight inspection",
            include_str!("update_platform_preflight.rs"),
        ),
    ] {
        assert!(
            source.contains("storage_error::"),
            "{name} must route storage failures through storage_error"
        );
        assert!(
            !source.contains("map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)"),
            "{name} must not discard durable error evidence"
        );
        assert!(
            !source.contains("Err(_) => Err(OllamaErrorCode::OllamaStorageUnavailable)"),
            "{name} must not discard inspection error evidence"
        );
    }
}
