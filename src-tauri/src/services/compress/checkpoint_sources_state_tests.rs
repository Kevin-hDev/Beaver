use crate::services::agent_local::types_session::SubagentLastActivity;

#[tokio::test]
async fn active_subagents_keep_mission_activity_and_identity() {
    use tokio_util::sync::CancellationToken;
    let parent = create_session("parent").await;
    let mut children = Vec::new();
    for index in 0..2 {
        let mut child = create_session(&format!("child-{index}")).await;
        child.parent_session_id = Some(parent.id.clone());
        child.subagent_type = Some("explorer".into());
        child.subagent_status = Some("running".into());
        child.subagent_prompt = Some(format!("mission-{index}"));
        child.subagent_last_activity = Some(SubagentLastActivity {
            kind: "tool".into(),
            label: "Reading".into(),
            detail: Some(format!("file-{index}")),
            updated_at: chrono::Utc::now(),
        });
        crate::services::agent_local::session_store::save(&child)
            .await
            .unwrap();
        crate::services::agent_local::subagent_registry::register(
            &parent.id,
            &child.id,
            CancellationToken::new(),
        )
        .await
        .unwrap();
        children.push(child);
    }
    let report = build_report(children[0].id.clone(), "report");
    crate::services::agent_local::subagent_hidden_reports::append(&parent.id, report)
        .await
        .unwrap();
    let reloaded = crate::services::agent_local::session_store::get(&parent.id)
        .await
        .unwrap();
    let state = super::super::checkpoint_subagents::collect(&reloaded, 2_000).await;

    assert_eq!(state.active.len(), 2);
    assert_eq!(state.pending_reports[0].child_session_id, children[0].id);
    assert!(state
        .active
        .iter()
        .all(|child| !child.mission.is_empty() && child.last_activity.is_some()));
    crate::services::agent_local::subagent_hidden_reports::acknowledge_reports(
        &parent.id,
        &[state.pending_reports[0].report_id.clone()],
    )
    .await
    .unwrap();
    let delivered_parent = crate::services::agent_local::session_store::get(&parent.id)
        .await
        .unwrap();
    let delivered = super::super::checkpoint_subagents::collect(&delivered_parent, 2_000).await;
    assert_eq!(delivered.delivered_report_ids.len(), 1);

    for child in &children {
        crate::services::agent_local::subagent_registry::unregister(&child.id).await;
        crate::services::agent_local::session_store::delete_one(&child.id)
            .await
            .unwrap();
    }
    crate::services::agent_local::session_store::delete_one(&parent.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn seventeenth_report_waits_durably_then_moves_into_a_delivered_slot() {
    let parent = create_session("overflow-parent").await;
    let mut first_id = String::new();
    for index in 0..16 {
        let report = build_report(uuid::Uuid::new_v4().to_string(), "done");
        if index == 0 {
            first_id = report.id.clone();
        }
        crate::services::agent_local::subagent_hidden_reports::append(&parent.id, report)
            .await
            .unwrap();
    }
    let overflow = build_report(uuid::Uuid::new_v4().to_string(), "seventeenth");
    crate::services::agent_local::subagent_report_overflow::enqueue(&parent.id, overflow.clone())
        .await
        .unwrap();
    let full = crate::services::agent_local::session_store::get(&parent.id)
        .await
        .unwrap();
    let checkpoint = super::super::checkpoint_subagents::collect(&full, 2_000).await;
    assert_eq!(checkpoint.pending_reports.len(), 17);
    assert!(checkpoint
        .pending_reports
        .iter()
        .any(|report| report.report_id == overflow.id));

    crate::services::agent_local::subagent_hidden_reports::acknowledge_reports(
        &parent.id,
        std::slice::from_ref(&first_id),
    )
    .await
    .unwrap();
    let pending =
        crate::services::agent_local::subagent_hidden_reports::peek_reports(&parent.id).await;
    assert_eq!(pending.len(), 16);
    assert!(pending.iter().any(|report| report.id == overflow.id));
    assert!(
        crate::services::agent_local::subagent_report_overflow::pending_for_parent(&parent.id)
            .await
            .is_empty()
    );
    let stored = crate::services::agent_local::session_store::get(&parent.id)
        .await
        .unwrap();
    assert!(!stored
        .subagent_hidden_reports
        .iter()
        .any(|report| report.id == first_id));
    crate::services::agent_local::session_store::delete_one(&parent.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn chatbot_live_state_excludes_git_plans_and_todos() {
    let mut session = create_session("chatbot").await;
    session.working_dir = "/private/not-read".into();
    session
        .todos
        .push(crate::services::agent_local::types_todo::AgentTodoItem {
            content: "agent-only".into(),
            active_form: None,
            status: crate::services::agent_local::types_todo::AgentTodoStatus::Pending,
        });
    let capabilities =
        super::super::session_capabilities::SessionCompressionCapabilities::from_runtime(
            true,
            &["todo_write".into(), "read_file".into()],
            true,
            true,
            true,
        )
        .unwrap();
    let state = super::super::checkpoint_live_state::collect(&session, &capabilities);
    assert!(state.git.is_empty());
    assert!(state.todos.is_empty());
    assert!(state.active_plan.is_none());
    crate::services::agent_local::session_store::delete_one(&session.id)
        .await
        .unwrap();
}

async fn create_session(name: &str) -> crate::services::agent_local::types_session::AgentSession {
    crate::services::agent_local::session_store::create_full(name, "model", "ollama", false, None)
        .await
        .unwrap()
}

fn build_report(
    child_id: String,
    summary: &str,
) -> crate::services::agent_local::types_session::SubagentHiddenReport {
    crate::services::agent_local::subagent_hidden_reports::build_report(
        child_id,
        "child".into(),
        "explorer".into(),
        "completed".into(),
        summary.into(),
    )
}
