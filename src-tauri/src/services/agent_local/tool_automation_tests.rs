use serde_json::{json, Value};

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
    let _guard = super::tool_automation::AUTOMATION_TOOL_TEST_LOCK
        .lock()
        .await;
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

    let listed = dispatch(&session.id, cwd, cancel.clone(), json!({"action":"list"})).await;
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
    assert!(crate::services::automations::read_all()
        .await
        .unwrap()
        .is_empty());

    super::session_store::delete_one(&session.id).await.unwrap();
    super::session_store::remove_session_lock(&session.id).await;
}

#[tokio::test]
async fn dispatcher_accepts_multiline_prompts_on_create_and_update() {
    let _guard = super::tool_automation::AUTOMATION_TOOL_TEST_LOCK
        .lock()
        .await;
    crate::services::automations::mutate(|items| {
        items.clear();
        Ok(())
    })
    .await
    .unwrap();
    let session = super::session_store::create_full(
        "Multiline automation",
        "gpt-5.6-luna",
        "codex-oauth",
        false,
        None,
    )
    .await
    .unwrap();
    let cancel = tokio_util::sync::CancellationToken::new();
    let created = dispatch(
        &session.id,
        std::path::Path::new("."),
        cancel.clone(),
        json!({
            "action":"create",
            "name":"CI",
            "description":"Étape 1\nÉtape 2",
            "prompt":"Vérifie la CI\nPuis résume les erreurs",
            "target_mode":"resume_session",
            "schedule":{"kind":"after_completion","delay_minutes":10}
        }),
    )
    .await;
    let id = created["data"]["id"].as_str().unwrap();
    assert_eq!(
        created["data"]["prompt"],
        "Vérifie la CI\nPuis résume les erreurs"
    );
    assert_eq!(created["data"]["description"], "Étape 1\nÉtape 2");

    let updated = dispatch(
        &session.id,
        std::path::Path::new("."),
        cancel,
        json!({
            "action":"update",
            "automation_id":id,
            "patch":{"prompt":"Relis les tests\nPuis publie le résultat"}
        }),
    )
    .await;
    assert_eq!(
        updated["data"]["prompt"],
        "Relis les tests\nPuis publie le résultat"
    );

    crate::services::automations::mutate(|items| {
        items.clear();
        Ok(())
    })
    .await
    .unwrap();
    super::session_store::delete_one(&session.id).await.unwrap();
    super::session_store::remove_session_lock(&session.id).await;
}

#[tokio::test]
async fn external_instruction_cannot_mutate_another_sessions_automation_without_manual_approval() {
    let _guard = super::tool_automation::AUTOMATION_TOOL_TEST_LOCK
        .lock()
        .await;
    crate::services::automations::mutate(|items| {
        items.clear();
        Ok(())
    })
    .await
    .unwrap();
    let owner =
        super::session_store::create_full("Owner", "gpt-5.6-luna", "codex-oauth", false, None)
            .await
            .unwrap();
    let caller =
        super::session_store::create_full("Caller", "gpt-5.6-luna", "codex-oauth", false, None)
            .await
            .unwrap();
    let created = dispatch(
        &owner.id,
        std::path::Path::new("."),
        tokio_util::sync::CancellationToken::new(),
        json!({
            "action":"create",
            "name":"CI",
            "prompt":"Vérifie",
            "target_mode":"resume_session",
            "schedule":{"kind":"after_completion","delay_minutes":10}
        }),
    )
    .await;
    let id = created["data"]["id"].as_str().unwrap();
    for (request_id, args) in [
        (
            "external-instruction-update",
            json!({"action":"update","automation_id":id,"patch":{"name":"Piratée"}}),
        ),
        (
            "external-instruction-delete",
            json!({"action":"delete","automation_id":id}),
        ),
    ] {
        let cancel = tokio_util::sync::CancellationToken::new();
        cancel.cancel();
        let result = super::tool_executor_write::execute_write(
            &super::stream_events::AgentEventEmitter::test(caller.id.clone()),
            "manage_automation",
            &args,
            std::path::Path::new("."),
            "manual",
            &mut super::write_guard::WriteGuard::new(),
            &caller.id,
            request_id,
            cancel,
            false,
            None,
        )
        .await;
        assert!(result.is_error);
    }

    let remaining = crate::services::automations::read_all().await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].name, "CI");
    for session in [&owner.id, &caller.id] {
        super::session_store::delete_one(session).await.unwrap();
        super::session_store::remove_session_lock(session).await;
    }
    crate::services::automations::mutate(|items| {
        items.clear();
        Ok(())
    })
    .await
    .unwrap();
}

async fn dispatch(
    session_id: &str,
    cwd: &std::path::Path,
    cancel: tokio_util::sync::CancellationToken,
    args: Value,
) -> Value {
    let result =
        super::tool_dispatcher::dispatch("manage_automation", &args, cwd, session_id, cancel).await;
    assert!(!result.is_error, "{}", result.content);
    serde_json::from_str(&result.content).unwrap()
}
