use super::create_heartbeat_session;
use crate::models::{ScheduledWakeup, WakeupSchedule};
use crate::services::agent_local::session_store;
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, NonReplayTarget, ReasoningModeId, RouteId,
};

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
async fn heartbeat_session_defers_prompt_persistence_to_conversation_admission() {
    let session_id = create_heartbeat_session(&wakeup(None))
        .await
        .expect("create heartbeat session");
    let session = session_store::get(&session_id)
        .await
        .expect("reload heartbeat session");

    assert!(session.messages.is_empty());

    crate::services::scheduler::admit_wakeup_turn(
        &session_id,
        "Inspecte le projet",
        ContinuationTarget::Forbidden(NonReplayTarget {
            route_id: RouteId::Ollama,
            model_id: "test-model".into(),
            reasoning_mode: ReasoningModeId::Off,
        }),
    )
    .await
    .expect("persist prompt through admission");

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
