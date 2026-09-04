use super::*;
use crate::services::agent_local::tool_artifact::{ArtifactPurpose, PendingArtifact};
use crate::services::agent_local::types_tools::{ToolFileChange, ToolFileChangeStatus};

#[test]
fn truncation_turns_a_success_into_a_partial_result() {
    let mut result = ToolResult::ok("preview");

    result.mark_truncated(true);

    assert_eq!(result.status, ToolResultStatus::Partial);
    assert!(!result.is_error);
    assert!(result.truncated);
}

#[test]
fn truncation_preserves_an_error_status() {
    let mut result = ToolResult::execution("test_failure", "preview", false);

    result.mark_truncated(true);

    assert_eq!(result.status, ToolResultStatus::Error);
    assert!(result.is_error);
    assert!(result.truncated);
}

#[test]
fn cancellation_preserves_result_context() {
    let result = ToolResult::cancelled("cancelled after a partial write")
        .with_affected_paths(vec!["changed.txt".to_string()]);

    assert_eq!(result.status, ToolResultStatus::Cancelled);
    assert_eq!(result.content, "cancelled after a partial write");
    assert_eq!(result.affected_paths(), ["changed.txt"]);
    assert_eq!(
        result.error.unwrap().category,
        ToolErrorCategory::Cancelled
    );
}

#[test]
fn warnings_remove_unsafe_controls() {
    let result = ToolResult::ok("ok").with_warning("safe\u{202e}text\0");

    assert_eq!(result.warnings, ["safetext"]);
}

#[test]
fn result_metadata_stays_flat_and_the_error_variant_stays_small() {
    let original = ToolResult::error(
        "failed",
        "test_failure",
        ToolErrorCategory::Execution,
        false,
    )
    .with_warning("warning");

    let serialized = serde_json::to_value(&original).unwrap();
    assert_eq!(serialized["status"], "error");
    assert_eq!(serialized["warnings"][0], "warning");
    assert!(serialized.get("details").is_none());

    let restored: ToolResult = serde_json::from_value(serialized).unwrap();
    assert_eq!(restored.status, ToolResultStatus::Error);
    assert_eq!(restored.error.unwrap().code.as_ref(), "test_failure");
    assert!(
        std::mem::size_of::<ToolResult>() <= 128,
        "ToolResult must remain small enough for Result error variants"
    );
}

#[test]
fn file_change_details_keep_a_bounded_sample_and_report_counts() {
    let changes = (0..200)
        .map(|index| ToolFileChange {
            path: format!("/repo/{index}.txt"),
            status: ToolFileChangeStatus::Added,
            additions: 1,
            deletions: 0,
            diff: None,
        })
        .collect();
    let mut result = ToolResult::ok("done").with_file_changes(changes);

    let counts = result.bound_file_changes();

    assert_eq!(counts, Some((200, 128)));
    assert_eq!(result.file_changes().len(), 128);
}

#[test]
fn affected_paths_keep_a_bounded_sample_and_report_counts() {
    let paths = (0..200).map(|index| format!("src/file-{index}.rs")).collect();
    let mut result = ToolResult::ok("done").with_affected_paths(paths);

    let counts = result.bound_affected_paths();

    assert_eq!(counts, Some((200, 128)));
    assert_eq!(result.affected_paths().len(), 128);
    assert_eq!(result.affected_paths()[0], "src/file-0.rs");
}

#[test]
fn pending_artifacts_are_bounded_hidden_and_taken_once() {
    let artifacts = (0..crate::services::extensions::types::MAX_RESULT_FILES)
        .map(|index| {
            PendingArtifact::from_validated(
                format!("result-{index}.txt"),
                Some(format!("Result {index}")),
                ArtifactPurpose::Artifact,
            )
        })
        .collect();
    let mut result = ToolResult::ok("done");

    assert!(result.set_pending_artifacts(artifacts).is_ok());
    assert_eq!(result.pending_artifacts().len(), 8);
    assert!(serde_json::to_value(&result)
        .unwrap()
        .get("pendingArtifacts")
        .is_none());
    assert_eq!(result.take_pending_artifacts().len(), 8);
    assert!(result.pending_artifacts().is_empty());
    assert!(result
        .set_pending_artifacts(vec![
            PendingArtifact::from_validated(
                "extra.txt".to_string(),
                Some("Extra".to_string()),
                ArtifactPurpose::Preview,
            );
            crate::services::extensions::types::MAX_RESULT_FILES + 1
        ])
        .is_err());
}
