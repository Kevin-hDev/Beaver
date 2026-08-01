use super::tool_list_dir::list_dir;
use super::tool_result_contract::ToolResultStatus;

#[tokio::test]
async fn empty_directory_is_an_explicit_success() {
    let directory = tempfile::tempdir().unwrap();
    let result = list_dir(".", directory.path()).await;

    assert_eq!(result.status, ToolResultStatus::Success);
    assert_eq!(result.content, "(dossier vide)");
}

#[tokio::test]
async fn a_file_cannot_be_reported_as_a_successful_directory_listing() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("file.txt"), "data").unwrap();

    let result = list_dir("file.txt", directory.path()).await;

    assert!(result.is_error);
    assert_eq!(
        result.error.as_ref().unwrap().code.as_ref(),
        "path_not_directory"
    );
}

#[tokio::test]
async fn a_missing_directory_has_a_not_found_code() {
    let directory = tempfile::tempdir().unwrap();

    let result = list_dir("missing", directory.path()).await;

    assert_eq!(
        result.error.as_ref().unwrap().code.as_ref(),
        "directory_not_found"
    );
}

#[tokio::test]
async fn an_exactly_full_directory_is_not_reported_as_truncated() {
    let directory = tempfile::tempdir().unwrap();
    for index in 0..500 {
        std::fs::write(directory.path().join(format!("{index:03}.txt")), "").unwrap();
    }

    let result = list_dir(".", directory.path()).await;

    assert_eq!(result.status, ToolResultStatus::Success);
    assert!(!result.truncated);
    assert_eq!(result.content.lines().count(), 500);
}

#[tokio::test]
async fn large_directories_report_a_bounded_partial_result() {
    let directory = tempfile::tempdir().unwrap();
    for index in 0..501 {
        std::fs::write(directory.path().join(format!("{index:03}.txt")), "").unwrap();
    }

    let result = list_dir(".", directory.path()).await;

    assert_eq!(result.status, ToolResultStatus::Partial);
    assert!(result.truncated);
    assert!(result.warnings.iter().any(|warning| warning.contains("500")));
    assert!(result.content.lines().count() <= 500);
}
