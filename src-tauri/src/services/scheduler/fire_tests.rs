use super::{create_heartbeat_session, persist_agent_result, persisted_agent_messages};
use crate::models::{ScheduledWakeup, WakeupSchedule};
use crate::services::agent_local::session_store;
use crate::services::agent_local::types_ollama::{ChatMessage, ToolCallFunction, ToolCallOllama};
use crate::services::scheduler::agentic::ScheduledAgentResult;

fn wakeup(project_id: Option<String>) -> ScheduledWakeup {
    ScheduledWakeup {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Test scheduler session".into(),
        model: "test-model".into(),
        provider: "ollama".into(),
        prompt: "Inspecte le projet".into(),
        schedule: WakeupSchedule::Once {
            datetime: "2026-08-22T12:00".into(),
        },
        description: String::new(),
        project_id,
        active: true,
        paused_by_global: false,
        created_at: "2026-08-22T10:00:00Z".into(),
    }
}

#[tokio::test]
async fn heartbeat_session_contains_the_prompt_before_agent_start() {
    let session_id = create_heartbeat_session(&wakeup(None))
        .await
        .expect("create heartbeat session");
    let session = session_store::get(&session_id)
        .await
        .expect("reload heartbeat session");

    assert_eq!(session.messages.len(), 1);
    assert_eq!(session.messages[0].role, "user");
    assert_eq!(session.messages[0].content, "Inspecte le projet");

    let resolved = crate::commands::agent_working_dir::resolve_for_session(&session_id, None)
        .await
        .expect("resolve projectless workspace from persisted prompt");
    let workspace_root = resolved
        .path
        .parent()
        .expect("workspace root")
        .to_path_buf();

    session_store::delete_one(&session_id)
        .await
        .expect("delete heartbeat session");
    tokio::fs::remove_dir_all(workspace_root)
        .await
        .expect("delete test workspace");
}

#[tokio::test]
async fn heartbeat_session_rejects_a_project_removed_before_execution() {
    let result = create_heartbeat_session(&wakeup(Some(format!(
        "missing-project-{}",
        uuid::Uuid::new_v4()
    ))))
    .await;

    if let Ok(session_id) = &result {
        session_store::delete_one(session_id)
            .await
            .expect("delete unexpected heartbeat session");
    }

    assert!(result.is_err());
}

#[test]
fn persistence_keeps_only_messages_produced_after_the_scheduled_prompt() {
    let completed = vec![
        ChatMessage {
            role: "system".into(),
            content: "Contexte Beaver".into(),
            ..Default::default()
        },
        ChatMessage {
            role: "user".into(),
            content: "Inspecte le projet".into(),
            ..Default::default()
        },
        ChatMessage {
            role: "assistant".into(),
            content: "Je vérifie.".into(),
            ..Default::default()
        },
        ChatMessage {
            role: "tool".into(),
            content: "README.md".into(),
            tool_name: Some("list_dir".into()),
            ..Default::default()
        },
        ChatMessage {
            role: "assistant".into(),
            content: "Terminé.".into(),
            ..Default::default()
        },
    ];

    let persisted = persisted_agent_messages(&completed);
    let roles = persisted
        .iter()
        .map(|message| message.role.as_str())
        .collect::<Vec<_>>();

    assert_eq!(roles, vec!["assistant", "tool", "assistant"]);
}

#[tokio::test]
async fn tool_trace_is_persisted_before_missing_text_is_reported() {
    let session_id = create_heartbeat_session(&wakeup(None))
        .await
        .expect("create heartbeat session");
    let result = ScheduledAgentResult {
        messages: vec![
            ChatMessage {
                role: "user".into(),
                content: "Inspecte le projet".into(),
                ..Default::default()
            },
            ChatMessage {
                role: "assistant".into(),
                tool_calls: Some(vec![ToolCallOllama {
                    id: Some("call-1".into()),
                    extra_content: None,
                    function: ToolCallFunction {
                        name: "list_dir".into(),
                        arguments: serde_json::json!({"path": "."}),
                    },
                }]),
                ..Default::default()
            },
            ChatMessage {
                role: "tool".into(),
                content: "README.md".into(),
                tool_name: Some("list_dir".into()),
                ..Default::default()
            },
        ],
        tokens: 12,
        has_text_result: false,
    };

    let outcome = persist_agent_result(&session_id, result).await;
    let saved = session_store::get(&session_id)
        .await
        .expect("reload heartbeat session");
    let roles = saved
        .messages
        .iter()
        .map(|message| message.role.as_str())
        .collect::<Vec<_>>();

    assert!(outcome.is_err());
    assert_eq!(roles, vec!["user", "assistant", "tool"]);

    session_store::delete_one(&session_id)
        .await
        .expect("delete heartbeat session");
}
