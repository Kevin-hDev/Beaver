use crate::services::agent_local::{session_store, tool_automation, tool_dispatcher};
use crate::services::scheduler;
use chrono::{DateTime, Duration, Utc};
use serde_json::{json, Value};
use std::path::Path;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[tokio::test]
async fn tool_lifecycle_resumes_history_and_self_deletes() {
    let _guard = tool_automation::AUTOMATION_TOOL_TEST_LOCK.lock().await;
    super::mutate(|items| {
        items.clear();
        Ok(())
    })
    .await
    .unwrap();
    let session = session_store::create_full(
        "Automation lifecycle",
        "gpt-5.6-luna",
        "codex-oauth",
        false,
        None,
    )
    .await
    .unwrap();
    let created = dispatch(
        &session.id,
        json!({
            "action":"create", "name":"CI", "prompt":"Vérifie la CI",
            "target_mode":"resume_session",
            "schedule":{"kind":"after_completion","delay_minutes":10}
        }),
    )
    .await;
    let id = Uuid::parse_str(created["data"]["id"].as_str().unwrap()).unwrap();

    assert_eq!(
        dispatch(&session.id, json!({"action":"list"})).await["data"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        dispatch(&session.id, json!({"action":"get","automation_id":id})).await["data"]
            ["target_session_id"],
        session.id
    );
    let updated = dispatch(
        &session.id,
        json!({"action":"update","automation_id":id,"patch":{"model":"gpt-5.6-sol"}}),
    )
    .await;
    assert_eq!(updated["data"]["model"], "gpt-5.6-sol");

    let definition = definition(id).await;
    let scheduled_for = next_fire(&definition);
    let occurrence_id = ready_occurrence(&definition, scheduled_for).await;
    let (target_id, created_target) = scheduler::fire::target_session_for_test(&definition)
        .await
        .unwrap();
    assert_eq!(target_id, session.id);
    assert!(!created_target);
    finish(occurrence_id, &session.id, scheduled_for).await;

    let history = dispatch(
        &session.id,
        json!({"action":"history","automation_id":id,"limit":10}),
    )
    .await;
    assert_eq!(history["data"]["items"][0]["status"], "ok");
    let actor = super::AutomationActor {
        origin: super::AutomationOrigin::Session,
        session_or_channel_id: session.id.clone(),
        current_automation_id: Some(id),
    };
    let _actor = super::actor_context::register_actor("automation-e2e-delete", actor).unwrap();
    let deleted = tool_dispatcher::dispatch_for_mode(
        "manage_automation",
        &json!({"action":"delete","automation_id":id}),
        Path::new("."),
        &session.id,
        Some("automation-e2e-delete"),
        CancellationToken::new(),
        false,
    )
    .await;
    assert!(!deleted.is_error, "{}", deleted.content);
    assert!(super::read_all()
        .await
        .unwrap()
        .iter()
        .all(|item| item.id != id));
    assert!(
        scheduler::runtime_for_test(&crate::services::paths::data_dir())
            .await
            .unwrap()
            .occurrences
            .iter()
            .all(|item| item.automation_id != id)
    );
    remove_session(&session.id).await;
}

#[tokio::test]
async fn cron_new_session_does_not_write_to_its_creator() {
    let _guard = tool_automation::AUTOMATION_TOOL_TEST_LOCK.lock().await;
    let creator = session_store::create_full(
        "Automation creator",
        "gpt-5.6-luna",
        "codex-oauth",
        false,
        None,
    )
    .await
    .unwrap();
    let created = dispatch(
        &creator.id,
        json!({
            "action":"create", "name":"Veille", "prompt":"Vérifie",
            "target_mode":"new_session",
            "schedule":{"kind":"cron","expression":"* * * * *","timezone":"UTC"}
        }),
    )
    .await;
    let id = Uuid::parse_str(created["data"]["id"].as_str().unwrap()).unwrap();
    let definition = definition(id).await;
    let scheduled_for = next_fire(&definition);
    let occurrence_id = ready_occurrence(&definition, scheduled_for).await;
    let (target_id, created_target) = scheduler::fire::target_session_for_test(&definition)
        .await
        .unwrap();
    assert!(created_target);
    assert!(session_store::get(&target_id).await.unwrap().is_heartbeat);
    assert!(session_store::get(&creator.id)
        .await
        .unwrap()
        .messages
        .is_empty());
    finish(occurrence_id, &target_id, scheduled_for).await;
    assert_eq!(
        dispatch(
            &creator.id,
            json!({"action":"history","automation_id":id,"limit":10}),
        )
        .await["data"]["items"][0]["session_id"],
        target_id
    );
    dispatch(&creator.id, json!({"action":"delete","automation_id":id})).await;
    remove_session(&target_id).await;
    remove_session(&creator.id).await;
}

pub(super) async fn dispatch(session_id: &str, args: Value) -> Value {
    let result = tool_dispatcher::dispatch(
        "manage_automation",
        &args,
        Path::new("."),
        session_id,
        CancellationToken::new(),
    )
    .await;
    assert!(!result.is_error, "{}", result.content);
    serde_json::from_str(&result.content).unwrap()
}

async fn definition(id: Uuid) -> crate::models::AutomationDefinition {
    super::read_all()
        .await
        .unwrap()
        .into_iter()
        .find(|item| item.id == id)
        .unwrap()
}

fn next_fire(definition: &crate::models::AutomationDefinition) -> DateTime<Utc> {
    super::next_fire::next_fire_at(definition, definition.created_at)
        .unwrap()
        .unwrap()
        .at
}

async fn ready_occurrence(
    definition: &crate::models::AutomationDefinition,
    scheduled_for: DateTime<Utc>,
) -> Uuid {
    match scheduler::admit_due_for_test(
        &crate::services::paths::data_dir(),
        definition,
        scheduled_for,
        scheduled_for,
    )
    .await
    .unwrap()
    {
        super::RuntimeAdmission::Ready { occurrence_id } => occurrence_id,
        _ => panic!("the occurrence must be ready"),
    }
}

async fn finish(occurrence_id: Uuid, session_id: &str, scheduled_for: DateTime<Utc>) {
    let root = crate::services::paths::data_dir();
    scheduler::mark_running_for_test(&root, occurrence_id, scheduled_for)
        .await
        .unwrap();
    scheduler::mark_terminal_for_test(
        &root,
        occurrence_id,
        run_result(session_id, scheduled_for + Duration::seconds(1)),
    )
    .await
    .unwrap();
    scheduler::publish_terminal_for_test(&root, occurrence_id)
        .await
        .unwrap();
}

pub(super) fn run_result(session_id: &str, finished_at: DateTime<Utc>) -> super::OccurrenceResult {
    super::OccurrenceResult {
        status: super::OccurrenceResultStatus::Ok,
        finished_at,
        error_code: None,
        session_id: Some(session_id.into()),
        tokens: Some(1),
        missed_count: None,
        first_scheduled_for: None,
        last_scheduled_for: None,
    }
}

async fn remove_session(session_id: &str) {
    session_store::delete_one(session_id).await.unwrap();
    session_store::remove_session_lock(session_id).await;
}
