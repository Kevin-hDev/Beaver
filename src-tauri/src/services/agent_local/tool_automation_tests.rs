use serde_json::{json, Value};

static TOOL_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn valid_requests() -> Vec<Value> {
    vec![
        json!({"action":"list"}),
        json!({"action":"get","automation_id":uuid::Uuid::new_v4()}),
        json!({
            "action":"create",
            "name":"Vérification CI",
            "prompt":"Vérifie la CI.",
            "target_mode":"resume_session",
            "schedule":{"kind":"after_completion","delay_minutes":10}
        }),
        json!({
            "action":"update",
            "automation_id":uuid::Uuid::new_v4(),
            "patch":{"status":"disabled"}
        }),
        json!({"action":"history","automation_id":uuid::Uuid::new_v4(),"limit":20}),
        json!({"action":"delete","automation_id":uuid::Uuid::new_v4()}),
    ]
}

#[test]
fn six_strict_action_shapes_are_accepted() {
    for request in valid_requests() {
        let generic = super::tool_validate::validate("manage_automation", &request).unwrap();
        assert!(super::tool_automation_validation::parse(&generic).is_ok());
    }
}

#[test]
fn unknown_cross_action_confirmation_and_identity_fields_are_rejected() {
    for request in [
        json!({"action":"list","unknown":true}),
        json!({"action":"list","automation_id":uuid::Uuid::new_v4()}),
        json!({"action":"delete","automation_id":uuid::Uuid::new_v4(),"confirm":true}),
        json!({"action":"list","creator_session_id":"forged"}),
        json!({"action":"list","origin":"forged"}),
        json!({"action":"list","current_automation_id":uuid::Uuid::new_v4()}),
    ] {
        let result = super::tool_validate::validate("manage_automation", &request)
            .map_err(|_| ())
            .and_then(|cleaned| {
                super::tool_automation_validation::parse(&cleaned)
                    .map(|_| cleaned)
                    .map_err(|_| ())
            });
        assert!(result.is_err());
    }
}

#[test]
fn nested_schedule_and_patch_are_strict() {
    for request in [
        json!({
            "action":"create",
            "name":"CI",
            "prompt":"Vérifie",
            "target_mode":"new_session",
            "schedule":{"kind":"after_completion","delay_minutes":10,"extra":true}
        }),
        json!({
            "action":"update",
            "automation_id":uuid::Uuid::new_v4(),
            "patch":{"status":"disabled","target_mode":"resume_session"}
        }),
    ] {
        let generic = super::tool_validate::validate("manage_automation", &request).unwrap();
        assert!(super::tool_automation_validation::parse(&generic).is_err());
    }
}

#[test]
fn schema_is_strict_and_does_not_advertise_legacy_fields() {
    let definition = super::tool_definitions_automation::automation_definition();
    let encoded = definition.to_string();

    assert!(encoded.contains("oneOf"));
    for forbidden in ["confirm", "skill_ids", "tool_names", "creator_session_id"] {
        assert!(!encoded.contains(forbidden));
    }
}

#[tokio::test]
async fn dispatcher_runs_the_full_contract_and_allows_self_deletion() {
    let _guard = TOOL_TEST_LOCK.lock().await;
    crate::services::automations::mutate(|items| {
        items.clear();
        Ok(())
    })
    .await
    .unwrap();
    let session = super::session_store::create_full(
        "Automation tool fixture",
        "gpt-5.6-luna",
        "codex-oauth",
        false,
        Some("project-a".into()),
    )
    .await
    .unwrap();
    let cwd = std::path::Path::new(".");
    let cancel = tokio_util::sync::CancellationToken::new();

    let created = dispatch(
        &session.id,
        cwd,
        cancel.clone(),
        json!({
            "action":"create",
            "name":"CI",
            "prompt":"Vérifie la CI",
            "target_mode":"resume_session",
            "schedule":{"kind":"after_completion","delay_minutes":10}
        }),
    )
    .await;
    let id = created["data"]["id"].as_str().unwrap();
    assert_eq!(created["data"]["provider"], "codex-oauth");
    assert_eq!(created["data"]["model"], "gpt-5.6-luna");
    assert_eq!(created["data"]["target_session_id"], session.id);

    let listed = dispatch(
        &session.id,
        cwd,
        cancel.clone(),
        json!({"action":"list"}),
    )
    .await;
    assert_eq!(listed["data"].as_array().unwrap().len(), 1);
    assert!(listed.to_string().find("Vérifie la CI").is_none());
    assert!(!listed.to_string().contains("working_dir"));
    assert!(!listed.to_string().contains("/Users/"));

    let fetched = dispatch(
        &session.id,
        cwd,
        cancel.clone(),
        json!({"action":"get","automation_id":id}),
    )
    .await;
    assert_eq!(fetched["data"]["prompt"], "Vérifie la CI");
    assert!(!fetched.to_string().contains("working_dir"));
    assert!(!fetched.to_string().contains("/Users/"));

    let updated = dispatch(
        &session.id,
        cwd,
        cancel.clone(),
        json!({"action":"update","automation_id":id,"patch":{"name":"CI verte"}}),
    )
    .await;
    assert_eq!(updated["data"]["name"], "CI verte");

    let history = dispatch(
        &session.id,
        cwd,
        cancel.clone(),
        json!({"action":"history","automation_id":id,"limit":10}),
    )
    .await;
    assert_eq!(history["data"]["items"], json!([]));

    let automation_id = uuid::Uuid::parse_str(id).unwrap();
    let actor = crate::services::automations::AutomationActor {
        origin: crate::services::automations::AutomationOrigin::Session,
        session_or_channel_id: session.id.clone(),
        current_automation_id: Some(automation_id),
    };
    let _actor_guard = crate::services::automations::actor_context::register_actor(
        "automation-tool-self-delete",
        actor,
    )
    .unwrap();
    let deleted = super::tool_dispatcher::dispatch_for_mode(
        "manage_automation",
        &json!({"action":"delete","automation_id":id}),
        cwd,
        &session.id,
        Some("automation-tool-self-delete"),
        cancel,
        false,
    )
    .await;
    let deleted: Value = serde_json::from_str(&deleted.content).unwrap();
    assert_eq!(deleted["ok"], true);
    assert!(crate::services::automations::read_all().await.unwrap().is_empty());

    super::session_store::delete_one(&session.id).await.unwrap();
    super::session_store::remove_session_lock(&session.id).await;
}

async fn dispatch(
    session_id: &str,
    cwd: &std::path::Path,
    cancel: tokio_util::sync::CancellationToken,
    args: Value,
) -> Value {
    let result = super::tool_dispatcher::dispatch(
        "manage_automation",
        &args,
        cwd,
        session_id,
        cancel,
    )
    .await;
    assert!(!result.is_error, "{}", result.content);
    serde_json::from_str(&result.content).unwrap()
}
