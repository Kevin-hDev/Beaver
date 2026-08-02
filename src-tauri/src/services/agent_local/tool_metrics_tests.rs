use super::{build_report, update_entries, ToolMetricEntry, MAX_TRACKED_TOOLS};
use crate::services::agent_local::tool_result_contract::ToolErrorCategory;
use crate::services::agent_local::types_tools::ToolResult;

#[test]
fn outcomes_keep_failures_denials_policy_blocks_and_cancellations_distinct() {
    let mut entries = Vec::new();
    update_entries(&mut entries, "read_file", &ToolResult::ok("ok"), 1);
    update_entries(
        &mut entries,
        "read_file",
        &ToolResult::error(
            "denied",
            "user_denied_tool",
            ToolErrorCategory::Permission,
            false,
        ),
        2,
    );
    update_entries(
        &mut entries,
        "read_file",
        &ToolResult::error(
            "blocked",
            "write_guard_rejected",
            ToolErrorCategory::Permission,
            false,
        ),
        3,
    );
    update_entries(
        &mut entries,
        "read_file",
        &ToolResult::cancelled("cancelled"),
        4,
    );

    let metric = &entries[0];
    assert_eq!(metric.invocations, 4);
    assert_eq!(metric.success, 1);
    assert_eq!(metric.failed, 2);
    assert_eq!(metric.cancelled, 1);
    assert_eq!(metric.user_denied, 1);
    assert_eq!(metric.policy_blocked, 1);
    assert_eq!(metric.errors.permission, 2);
    assert_eq!(metric.errors.cancelled, 1);
}

#[test]
fn operating_system_permission_errors_are_not_called_policy_blocks() {
    let mut entries = Vec::new();
    update_entries(
        &mut entries,
        "read_file",
        &ToolResult::error(
            "unavailable",
            "file_permission_denied",
            ToolErrorCategory::Permission,
            false,
        ),
        1,
    );

    assert_eq!(entries[0].errors.permission, 1);
    assert_eq!(entries[0].policy_blocked, 0);
}

#[test]
fn externally_named_tool_collection_is_bounded_with_eviction() {
    let mut entries = Vec::new();
    for index in 0..=MAX_TRACKED_TOOLS {
        update_entries(
            &mut entries,
            &format!("extension.tool{index}"),
            &ToolResult::ok("ok"),
            index as i64,
        );
    }

    assert_eq!(entries.len(), MAX_TRACKED_TOOLS);
    assert!(!entries.iter().any(|entry| entry.name == "extension.tool0"));
    assert!(entries
        .iter()
        .any(|entry| entry.name == format!("extension.tool{MAX_TRACKED_TOOLS}")));
}

#[test]
fn report_is_bounded_and_ranks_failures_before_volume() {
    let mut entries = vec![metric("busy", 100, 0), metric("failing", 2, 2)];
    for index in 0..30 {
        entries.push(metric(&format!("tool{index}"), 1, 0));
    }

    let report = serde_json::to_value(build_report(entries, 100)).unwrap();
    assert_eq!(report["trackedTools"], 32);
    assert_eq!(report["tools"].as_array().unwrap().len(), 20);
    assert_eq!(report["tools"][0]["name"], "failing");
}

fn metric(name: &str, invocations: u64, failed: u64) -> ToolMetricEntry {
    ToolMetricEntry {
        name: name.to_string(),
        invocations,
        failed,
        ..Default::default()
    }
}
