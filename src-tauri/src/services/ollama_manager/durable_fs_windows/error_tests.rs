use super::super::OllamaFsErrorKind;
use super::{win_error, ERROR_ACCESS_DENIED};

#[test]
fn native_access_denied_keeps_its_numeric_code_and_classification() {
    let error = win_error(ERROR_ACCESS_DENIED);

    assert_eq!(error.kind(), OllamaFsErrorKind::PermissionDenied);
    assert_eq!(error.os_code(), Some(ERROR_ACCESS_DENIED));
}
