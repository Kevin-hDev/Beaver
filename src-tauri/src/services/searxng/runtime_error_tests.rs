use super::runtime_error::RuntimeError;

#[test]
fn runtime_errors_expose_only_fixed_categories_and_messages() {
    let cases = [
        (RuntimeError::ManifestInvalid, "manifest-invalid"),
        (RuntimeError::PythonUnavailable, "python-unavailable"),
        (
            RuntimeError::WheelhouseUnavailable,
            "wheelhouse-unavailable",
        ),
        (
            RuntimeError::EnvironmentUnavailable,
            "environment-unavailable",
        ),
    ];

    for (error, category) in cases {
        assert_eq!(error.category(), category);
        assert_eq!(error.public_code(), super::error_codes::RUNTIME_UNAVAILABLE);
    }
    assert_eq!(
        RuntimeError::Cancelled.public_code(),
        super::error_codes::SHUTTING_DOWN
    );
}
