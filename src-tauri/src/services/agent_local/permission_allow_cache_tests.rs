use super::{allowed_tool_count_for_test, clear_session, mark_allowed};

#[tokio::test]
async fn bounds_tools_remembered_for_one_session() {
    let session_id = uuid::Uuid::new_v4().to_string();

    for index in 0..32 {
        mark_allowed(&session_id, &format!("tool-{index}")).await;
    }

    assert!(allowed_tool_count_for_test(&session_id).await <= 16);
    clear_session(&session_id).await;
}

#[tokio::test]
async fn never_remembers_tools_excluded_from_session_allow() {
    let session_id = uuid::Uuid::new_v4().to_string();

    mark_allowed(&session_id, "bash").await;
    mark_allowed(&session_id, "search_mcp_tools").await;

    assert_eq!(allowed_tool_count_for_test(&session_id).await, 0);
    clear_session(&session_id).await;
}
