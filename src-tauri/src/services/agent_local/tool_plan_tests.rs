#[test]
fn clean_text_rejects_too_long_values() {
    let long = "a".repeat(super::MAX_TITLE_CHARS + 1);
    assert!(super::clean_text(&long, super::MAX_TITLE_CHARS).is_err());
}

#[test]
fn upsert_keeps_history_bounded_by_caller_order() {
    let now = chrono::Utc::now();
    let mut runs = Vec::new();
    for idx in 0..25 {
        super::super::tool_plan_storage::upsert_run(
            &mut runs,
            super::AgentPlanRun {
                id: format!("00000000-0000-0000-0000-{idx:012}"),
                title: format!("Plan {idx}"),
                status: super::AgentPlanStatus::AwaitingApproval,
                path: "x".into(),
                created_at: now,
                updated_at: now,
            },
        );
        runs.truncate(super::MAX_PLAN_RUNS);
    }
    assert_eq!(runs.len(), super::MAX_PLAN_RUNS);
    assert_eq!(runs[0].title, "Plan 24");
}
