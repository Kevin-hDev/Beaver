use super::tool_glob::glob_files;
use super::tool_grep::grep;
use super::tool_result_contract::{ToolErrorCategory, ToolResultStatus};

#[tokio::test]
async fn invalid_search_patterns_have_validation_codes() {
    let directory = tempfile::tempdir().unwrap();
    let grep_result = grep("[", None, None, directory.path()).await;
    let glob_result = glob_files("[", None, directory.path()).await;

    for result in [grep_result, glob_result] {
        assert_eq!(result.status, ToolResultStatus::Error);
        assert_eq!(
            result.error.as_ref().unwrap().category,
            ToolErrorCategory::Validation
        );
    }
}

#[tokio::test]
async fn glob_limit_is_machine_readable() {
    let directory = tempfile::tempdir().unwrap();
    for index in 0..101 {
        std::fs::write(directory.path().join(format!("{index:03}.txt")), "").unwrap();
    }

    let result = glob_files("*.txt", None, directory.path()).await;

    assert_eq!(result.status, ToolResultStatus::Partial);
    assert!(result.truncated);
    assert!(result.content.contains("tronqué à 100"));
}

#[tokio::test]
async fn exact_glob_limit_is_not_reported_as_truncated() {
    let directory = tempfile::tempdir().unwrap();
    for index in 0..100 {
        std::fs::write(directory.path().join(format!("{index:03}.txt")), "").unwrap();
    }

    let result = glob_files("*.txt", None, directory.path()).await;

    assert_eq!(result.status, ToolResultStatus::Success);
    assert!(!result.truncated);
}

#[tokio::test]
async fn exact_grep_limit_is_not_reported_as_truncated() {
    let directory = tempfile::tempdir().unwrap();
    let lines = std::iter::repeat_n("match", 250).collect::<Vec<_>>().join("\n");
    std::fs::write(directory.path().join("matches.txt"), lines).unwrap();

    let result = grep("match", None, None, directory.path()).await;

    assert_eq!(result.status, ToolResultStatus::Success);
    assert!(!result.truncated);
}

#[tokio::test]
async fn grep_reports_truncation_only_after_an_extra_match() {
    let directory = tempfile::tempdir().unwrap();
    let lines = std::iter::repeat_n("match", 251).collect::<Vec<_>>().join("\n");
    std::fs::write(directory.path().join("matches.txt"), lines).unwrap();

    let result = grep("match", None, None, directory.path()).await;

    assert_eq!(result.status, ToolResultStatus::Partial);
    assert!(result.truncated);
}

#[tokio::test]
async fn no_match_remains_a_clean_success() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("file.txt"), "hello").unwrap();

    let result = grep("absent", None, None, directory.path()).await;

    assert_eq!(result.status, ToolResultStatus::Success);
    assert_eq!(result.content, "(aucun résultat)");
}

#[tokio::test]
async fn a_missing_search_root_is_not_reported_as_a_scan_failure() {
    let directory = tempfile::tempdir().unwrap();

    let result = glob_files("*.txt", Some("missing"), directory.path()).await;

    assert_eq!(
        result.error.as_ref().unwrap().code.as_ref(),
        "search_root_not_found"
    );
    assert_eq!(
        result.error.unwrap().category,
        ToolErrorCategory::NotFound
    );
}
