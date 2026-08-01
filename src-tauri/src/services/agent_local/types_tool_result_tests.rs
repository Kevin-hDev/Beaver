use super::*;

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
    let mut result = ToolResult::err("preview");

    result.mark_truncated(true);

    assert_eq!(result.status, ToolResultStatus::Error);
    assert!(result.is_error);
    assert!(result.truncated);
}

#[test]
fn cancellation_conversion_preserves_result_context() {
    let result = ToolResult::err("cancelled after a partial write")
        .with_affected_paths(vec!["changed.txt".to_string()])
        .into_cancelled();

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
