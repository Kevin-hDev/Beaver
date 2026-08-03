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

#[tokio::test]
async fn nested_directories_are_sorted_with_each_subtree_kept_together() {
    let directory = tempfile::tempdir().unwrap();
    let alpha = directory.path().join("alpha");
    let nested = alpha.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(alpha.join("first.txt"), "").unwrap();
    std::fs::write(nested.join("deep.txt"), "").unwrap();
    std::fs::write(directory.path().join("z-last.txt"), "").unwrap();

    let result = list_dir(".", directory.path()).await;

    assert_eq!(
        result.content,
        "alpha/\n  first.txt\n  nested/\n    deep.txt\nz-last.txt"
    );
}

#[test]
fn private_root_depth_uses_injected_access_roots() {
    let directory = tempfile::tempdir().unwrap();
    let private_input = directory.path().join("private");
    let project_input = directory.path().join("project");
    std::fs::create_dir(&private_input).unwrap();
    std::fs::create_dir(&project_input).unwrap();
    let private = dunce::canonicalize(private_input).unwrap();
    let project = dunce::canonicalize(project_input).unwrap();

    assert_eq!(
        super::tool_list_dir::listing_depth_in_roots(&private, &private, None),
        0
    );
    assert_eq!(
        super::tool_list_dir::listing_depth_in_roots(&private, &private, Some(&[])),
        0
    );
    assert_eq!(
        super::tool_list_dir::listing_depth_in_roots(
            &private,
            &private,
            Some(std::slice::from_ref(&private)),
        ),
        3
    );
    assert_eq!(
        super::tool_list_dir::listing_depth_in_roots(&project, &private, None),
        3
    );
}

#[test]
fn unavailable_private_root_keeps_its_original_path() {
    let directory = tempfile::tempdir().unwrap();
    let missing = directory.path().join("missing");

    assert_eq!(
        super::tool_list_dir::canonical_or_original(missing.clone()),
        missing
    );
}

#[tokio::test]
async fn zero_depth_lists_only_the_requested_directory() {
    let directory = tempfile::tempdir().unwrap();
    let nested = directory.path().join("nested");
    std::fs::create_dir(&nested).unwrap();
    std::fs::write(nested.join("hidden.txt"), "").unwrap();

    let result = super::tool_list_dir::list_resolved(directory.path().to_path_buf(), 0).await;

    assert_eq!(result.content, "nested/");
}

#[tokio::test]
async fn full_root_with_a_nonempty_directory_reports_omitted_descendants() {
    let directory = tempfile::tempdir().unwrap();
    let nested = directory.path().join("000-dir");
    std::fs::create_dir(&nested).unwrap();
    std::fs::write(nested.join("child.txt"), "").unwrap();
    for index in 1..500 {
        std::fs::write(directory.path().join(format!("{index:03}.txt")), "").unwrap();
    }

    let result = list_dir(".", directory.path()).await;

    assert!(result.truncated);
    assert_eq!(result.content.lines().count(), 500);
}
