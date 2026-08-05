use super::*;

#[test]
fn common_io_errors_have_distinct_categories() {
    let not_found = io_failure(
        std::io::Error::new(std::io::ErrorKind::NotFound, "missing"),
        "file_read_failed",
    );
    let directory = io_failure(
        std::io::Error::new(std::io::ErrorKind::IsADirectory, "directory"),
        "file_read_failed",
    );
    let timeout = io_failure(
        std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout"),
        "file_read_failed",
    );

    assert_eq!(not_found.error.unwrap().code.as_ref(), "file_not_found");
    assert_eq!(
        directory.error.unwrap().category,
        ToolErrorCategory::Validation
    );
    assert!(timeout.error.unwrap().retryable);
}

#[test]
fn interrupted_write_requires_verification_before_retry() {
    let result = io_failure(
        std::io::Error::new(std::io::ErrorKind::Interrupted, "interrupted"),
        "file_write_failed",
    );
    let error = result.error.unwrap();

    assert!(!error.retryable);
    assert!(error.hint.unwrap().contains("Vérifier le fichier"));
}

#[test]
fn missing_search_root_uses_the_platform_independent_not_found_code() {
    let result = search_root_failure(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "platform-specific message",
    ));
    let error = result.error.unwrap();

    assert_eq!(error.code.as_ref(), "search_root_not_found");
    assert_eq!(error.category, ToolErrorCategory::NotFound);
}
