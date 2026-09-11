use super::*;

#[tokio::test]
async fn ordinary_result_under_limit_is_unchanged() {
    let result = truncate_result(ToolResult::ok("small"), "bash", "unused").await;

    assert_eq!(result.content, "small");
    assert!(!result.truncated);
}

#[tokio::test]
async fn large_errors_are_bounded_and_the_full_result_is_retained() {
    let session_id = uuid::Uuid::new_v4().to_string();
    let full = "é".repeat(MAX_CHARS_ERROR + 1);
    let result = truncate_result(
        ToolResult::external("test_extension_failure", full.clone(), false),
        "extension",
        &session_id,
    )
    .await;

    assert!(result.is_error);
    assert!(result.truncated);
    assert!(result.content.contains("[Résultat tronqué"));
    let directory = data_dir().join("tool-results").join(&session_id);
    let files = std::fs::read_dir(&directory)
        .expect("persisted result directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("persisted result entries");
    assert_eq!(files.len(), 1);
    assert_eq!(std::fs::read_to_string(files[0].path()).unwrap(), full);
    let _ = std::fs::remove_dir_all(directory);
}

#[tokio::test]
async fn truncation_is_utf8_safe() {
    let session_id = uuid::Uuid::new_v4().to_string();
    let result = truncate_result(
        ToolResult::ok("🎉".repeat(MAX_CHARS_GLOB + 1)),
        "glob",
        &session_id,
    )
    .await;

    assert!(result.truncated);
    assert!(result.content.is_char_boundary(result.content.len()));
    let _ = std::fs::remove_dir_all(data_dir().join("tool-results").join(session_id));
}

#[tokio::test]
async fn read_file_at_the_limit_is_unchanged() {
    let content = "r".repeat(200_000);
    let result = truncate_result(ToolResult::ok(content.clone()), "read_file", "unused").await;

    assert_eq!(result.content, content);
    assert!(!result.truncated);
}

#[tokio::test]
async fn oversized_log_read_is_bounded_and_retained_outside_the_context() {
    let session_id = uuid::Uuid::new_v4().to_string();
    // The reported 707,488-character log was 260,347 OpenAI tokens. Character
    // bounds are deterministic here; provider token ratios are not.
    let full = (0..5_000)
        .map(|line| format!("{line}\t{}\n", "log entry ".repeat(14)))
        .collect::<String>();
    assert!(full.chars().count() > 700_000);

    let result = truncate_result(
        ToolResult::ok(full.clone()),
        "read_file",
        &session_id,
    )
    .await;

    assert!(result.truncated);
    assert!(result.content.chars().count() < 3_000);
    let directory = data_dir().join("tool-results").join(&session_id);
    let files = std::fs::read_dir(&directory)
        .expect("persisted result directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("persisted result entries");
    assert_eq!(files.len(), 1);
    assert_eq!(std::fs::read_to_string(files[0].path()).unwrap(), full);
    let _ = std::fs::remove_dir_all(directory);
}

#[tokio::test]
async fn read_file_over_the_limit_is_truncated_without_splitting_utf8() {
    let session_id = uuid::Uuid::new_v4().to_string();
    let result = truncate_result(
        ToolResult::ok(format!("{}🎉", "r".repeat(200_000))),
        "read_file",
        &session_id,
    )
    .await;

    assert!(result.truncated);
    assert!(result.content.is_char_boundary(result.content.len()));
    let _ = std::fs::remove_dir_all(data_dir().join("tool-results").join(session_id));
}

#[test]
fn persistence_failure_is_explicit_and_does_not_change_an_error_to_success() {
    let result = apply_truncation(
        ToolResult::execution("test_failure", "", false),
        "x".repeat(PREVIEW_SIZE),
        None,
        PREVIEW_SIZE + 1,
    );

    assert!(result.is_error);
    assert!(result.truncated);
    assert!(result.warnings[0].contains("pas pu être enregistré"));
}

#[tokio::test]
async fn result_storage_rejects_an_invalid_session_path() {
    assert!(persist_result("secret".into(), "../outside")
        .await
        .is_none());
}

#[tokio::test]
async fn persisted_result_path_is_directly_readable_by_the_file_tool() {
    let session_id = uuid::Uuid::new_v4().to_string();
    let full = "complete result\n".repeat(20_000);
    let path = persist_result(full, &session_id)
        .await
        .expect("persisted result path");
    assert!(std::path::Path::new(&path).is_absolute());

    let working_dir = tempfile::tempdir().unwrap();
    let read_result =
        super::super::tool_files::read_file(&path, working_dir.path(), 0, 50_000).await;
    assert!(!read_result.is_error);
    assert!(read_result.content.chars().count() > 200_000);

    let bounded = truncate_result(read_result, "read_file", &session_id).await;
    assert!(bounded.truncated);
    assert!(bounded.content.chars().count() < 3_000);

    let _ = std::fs::remove_dir_all(data_dir().join("tool-results").join(session_id));
}
