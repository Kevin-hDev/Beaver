use super::session_capabilities::SessionCompressionCapabilities;

#[test]
fn chatbot_keeps_only_observed_web_tools_and_no_agentic_category() {
    let capabilities = SessionCompressionCapabilities::from_runtime(
        true,
        &["web_search".into(), "web_fetch".into()],
        true,
        true,
        true,
    )
    .unwrap();

    assert_eq!(
        capabilities.tool_names.into_iter().collect::<Vec<_>>(),
        vec!["web_fetch", "web_search"]
    );
    assert!(!capabilities.project_context);
    assert!(!capabilities.subagents);
    assert!(!capabilities.git);
    assert!(!capabilities.plan_and_tasks);
}

#[test]
fn agentic_categories_follow_the_tools_really_sent() {
    let capabilities = SessionCompressionCapabilities::from_runtime(
        false,
        &[
            "read_file".into(),
            "delegate_task".into(),
            "plan_mode".into(),
        ],
        true,
        true,
        true,
    )
    .unwrap();

    assert!(capabilities.project_context);
    assert!(capabilities.subagents);
    assert!(capabilities.git);
    assert!(capabilities.plan_and_tasks);
}

#[test]
fn external_tool_collection_is_bounded() {
    let names = (0..257)
        .map(|index| format!("tool_{index}"))
        .collect::<Vec<_>>();
    assert!(SessionCompressionCapabilities::from_runtime(false, &names, true, true, true).is_err());
}

#[test]
fn existing_todos_remain_available_when_plan_mode_is_not_active() {
    let capabilities = SessionCompressionCapabilities::from_runtime(
        false,
        &["todo_write".into()],
        false,
        false,
        false,
    )
    .unwrap();
    assert!(capabilities.plan_and_tasks);
}
