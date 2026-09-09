use super::super::OllamaFsErrorKind;
use super::{
    win_error, ERROR_ACCESS_DENIED, ERROR_ALREADY_EXISTS, ERROR_FILE_EXISTS, ERROR_FILE_NOT_FOUND,
    ERROR_INVALID_PARAMETER, ERROR_LOCK_VIOLATION, ERROR_PATH_NOT_FOUND, ERROR_SHARING_VIOLATION,
};

#[test]
fn native_access_denied_keeps_its_numeric_code_and_classification() {
    let error = win_error(ERROR_ACCESS_DENIED);

    assert_eq!(error.kind(), OllamaFsErrorKind::PermissionDenied);
    assert_eq!(error.os_code(), Some(ERROR_ACCESS_DENIED));
}

#[test]
fn native_error_families_keep_their_classification() {
    for (codes, expected) in [
        (
            [ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND],
            OllamaFsErrorKind::NotFound,
        ),
        (
            [ERROR_ALREADY_EXISTS, ERROR_FILE_EXISTS],
            OllamaFsErrorKind::AlreadyExists,
        ),
        (
            [ERROR_SHARING_VIOLATION, ERROR_LOCK_VIOLATION],
            OllamaFsErrorKind::SharingViolation,
        ),
    ] {
        for code in codes {
            assert_eq!(win_error(code).kind(), expected);
        }
    }
    assert_eq!(
        win_error(ERROR_INVALID_PARAMETER).kind(),
        OllamaFsErrorKind::InvalidInput
    );
}
