use super::*;
use crate::models::{
    AutomationSchedule, AutomationStatus, AutomationTarget, WakeupRunErrorCode, WakeupRunStatus,
};
use chrono::{TimeZone, Utc};
use uuid::Uuid;

fn actor() -> AutomationActor {
    AutomationActor {
        origin: AutomationOrigin::Session,
        session_or_channel_id: "session-a".into(),
        current_automation_id: None,
    }
}

fn input(schedule: AutomationSchedule) -> CreateAutomation {
    CreateAutomation {
        name: "CI".into(),
        description: Some("Surveillance".into()),
        prompt: "Vérifie la CI".into(),
        target: AutomationTarget::ResumeSession {
            session_id: "session-a".into(),
        },
        provider: "codex-oauth".into(),
        model: "gpt-5.6-luna".into(),
        schedule,
        status: AutomationStatus::Active,
    }
}

#[tokio::test]
async fn create_list_get_update_delete_advances_revisions() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 10, 10, 0, 0).unwrap();
    let created = create_at(
        root.path(),
        &actor(),
        input(AutomationSchedule::AfterCompletion { delay_minutes: 10 }),
        now,
    )
    .await
    .unwrap();
    assert_eq!(created.definition.revision, 1);
    assert_eq!(
        created.definition.creator_session_id.as_deref(),
        Some("session-a")
    );
    assert_eq!(created.definition.anchor_at, Some(now));
    assert_eq!(list_at(root.path(), &actor(), now).await.unwrap().len(), 1);
    assert_eq!(
        get_at(root.path(), &actor(), created.definition.id, now)
            .await
            .unwrap()
            .definition,
        created.definition
    );

    let updated = update_at(
        root.path(),
        &actor(),
        created.definition.id,
        UpdateAutomation {
            name: Some("CI verte".into()),
            ..Default::default()
        },
        now,
    )
    .await
    .unwrap();
    assert_eq!(updated.definition.revision, 2);
    assert_eq!(updated.definition.anchor_at, Some(now));
    let rescheduled = update_at(
        root.path(),
        &actor(),
        created.definition.id,
        UpdateAutomation {
            schedule: Some(AutomationSchedule::AfterCompletion { delay_minutes: 20 }),
            ..Default::default()
        },
        now + chrono::Duration::minutes(1),
    )
    .await
    .unwrap();
    assert_eq!(
        rescheduled.definition.anchor_at,
        Some(now + chrono::Duration::minutes(1))
    );
    delete_at(root.path(), &actor(), created.definition.id)
        .await
        .unwrap();
    assert!(list_at(root.path(), &actor(), now)
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn multiline_prompts_and_descriptions_are_accepted() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 10, 10, 0, 0).unwrap();
    let mut request = input(AutomationSchedule::AfterCompletion { delay_minutes: 10 });
    request.prompt = "Vérifie la CI\n\tPuis résume les erreurs".into();
    request.description = Some("Étape 1\r\nÉtape 2".into());

    let created = create_at(root.path(), &actor(), request, now)
        .await
        .unwrap();

    assert!(created.definition.prompt.contains('\n'));
    assert!(created.definition.description.unwrap().contains("\r\n"));
}

#[tokio::test]
async fn list_reports_a_persisted_running_occurrence() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 10, 10, 0, 0).unwrap();
    let created = create_at(
        root.path(),
        &actor(),
        input(AutomationSchedule::AfterCompletion { delay_minutes: 10 }),
        now,
    )
    .await
    .unwrap();
    let admission = super::runtime_lifecycle::admit_at(root.path(), &created.definition, now, now)
        .await
        .unwrap();
    let super::runtime_lifecycle::RuntimeAdmission::Ready { occurrence_id } = admission else {
        panic!("occurrence should be ready");
    };
    super::runtime_lifecycle::mark_running_at(root.path(), occurrence_id, now)
        .await
        .unwrap();
    super::history_store::append_at(
        &root.path().join("logs/wakeups.jsonl"),
        HistoryEntry {
            run_id: Some(Uuid::new_v4()),
            automation_id: created.definition.id.to_string(),
            scheduled_for: now.to_rfc3339(),
            started_at: Some(now.to_rfc3339()),
            finished_at: now.to_rfc3339(),
            status: WakeupRunStatus::Error,
            error_code: Some(WakeupRunErrorCode::TargetSessionMissing),
            session_id: None,
            tokens: None,
            missed_count: None,
            first_scheduled_for: None,
            last_scheduled_for: None,
        },
    )
    .await
    .unwrap();

    let items = list_at(root.path(), &actor(), now).await.unwrap();
    assert!(items[0].running);
    assert_eq!(
        items[0].last_run.as_ref().unwrap().error_code,
        Some(WakeupRunErrorCode::TargetSessionMissing)
    );
}

#[tokio::test]
async fn immutable_fields_and_cross_provider_models_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc::now();
    let created = create_at(
        root.path(),
        &actor(),
        input(AutomationSchedule::AfterCompletion { delay_minutes: 5 }),
        now,
    )
    .await
    .unwrap();
    for patch in [
        UpdateAutomation {
            provider: Some("openai".into()),
            ..Default::default()
        },
        UpdateAutomation {
            creator_session_id: Some(Some("session-b".into())),
            ..Default::default()
        },
        UpdateAutomation {
            target: Some(AutomationTarget::NewSession { project_id: None }),
            ..Default::default()
        },
    ] {
        assert_eq!(
            update_at(root.path(), &actor(), created.definition.id, patch, now)
                .await
                .unwrap_err(),
            AutomationError::ImmutableField
        );
    }
    assert_eq!(
        update_at(
            root.path(),
            &actor(),
            created.definition.id,
            UpdateAutomation {
                model: Some("unknown-model".into()),
                ..Default::default()
            },
            now,
        )
        .await
        .unwrap_err(),
        AutomationError::ModelUnavailable
    );
    assert_eq!(
        update_at(
            root.path(),
            &actor(),
            created.definition.id,
            UpdateAutomation {
                status: Some(AutomationStatus::Completed),
                ..Default::default()
            },
            now,
        )
        .await
        .unwrap_err(),
        AutomationError::InvalidInput
    );
}

#[tokio::test]
async fn a_registered_model_without_tools_is_rejected() {
    let _catalog_guard = crate::services::llm::runtime_models::test_mutation_lock().await;
    let model = crate::services::llm::types::ModelInfo {
        id: "no-tools".into(),
        display_name: None,
        owned_by: None,
        context_length: None,
        max_output_tokens: None,
        supported_parameters: None,
        catalog_capabilities: Default::default(),
        supports_tools: false,
        supports_vision: false,
        supports_thinking: false,
        reasoning_contract: None,
        supports_fast_mode: false,
        reasoning_modes: Vec::new(),
        default_reasoning_mode: None,
        context_usage_includes_reasoning: true,
        is_free: false,
    };
    crate::services::llm::runtime_models::replace_provider("openai", &[model]).unwrap();
    let mut request = input(AutomationSchedule::AfterCompletion { delay_minutes: 5 });
    request.provider = "openai".into();
    request.model = "no-tools".into();
    assert_eq!(
        create_at(
            tempfile::tempdir().unwrap().path(),
            &actor(),
            request,
            Utc::now()
        )
        .await
        .unwrap_err(),
        AutomationError::ModelToolsUnsupported
    );
    crate::services::llm::runtime_models::replace_provider("openai", &[]).unwrap();
}

#[tokio::test]
async fn completion_and_reactivation_follow_the_current_schedule() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 10, 10, 0, 0).unwrap();
    let once = create_at(
        root.path(),
        &actor(),
        input(AutomationSchedule::Once {
            local_datetime: now.naive_utc() + chrono::Duration::hours(1),
            timezone: chrono_tz::UTC,
        }),
        now,
    )
    .await
    .unwrap();
    record_completion_at(root.path(), once.definition.id, now)
        .await
        .unwrap();
    record_completion_at(root.path(), once.definition.id, now)
        .await
        .unwrap();
    let completed = get_at(root.path(), &actor(), once.definition.id, now)
        .await
        .unwrap();
    assert_eq!(completed.definition.status, AutomationStatus::Completed);
    assert_eq!(completed.definition.revision, 2);
    assert_eq!(
        update_at(
            root.path(),
            &actor(),
            once.definition.id,
            UpdateAutomation {
                schedule: Some(AutomationSchedule::Once {
                    local_datetime: now.naive_utc() + chrono::Duration::hours(2),
                    timezone: chrono_tz::UTC,
                }),
                ..Default::default()
            },
            now,
        )
        .await
        .unwrap_err(),
        AutomationError::InvalidSchedule
    );
    let reactivated = update_at(
        root.path(),
        &actor(),
        once.definition.id,
        UpdateAutomation {
            schedule: Some(AutomationSchedule::AfterCompletion { delay_minutes: 15 }),
            status: Some(AutomationStatus::Active),
            ..Default::default()
        },
        now + chrono::Duration::minutes(1),
    )
    .await
    .unwrap();
    assert_eq!(
        reactivated.definition.anchor_at,
        Some(now + chrono::Duration::minutes(1))
    );

    let disabled = update_at(
        root.path(),
        &actor(),
        once.definition.id,
        UpdateAutomation {
            status: Some(AutomationStatus::Disabled),
            ..Default::default()
        },
        now + chrono::Duration::minutes(2),
    )
    .await
    .unwrap();
    record_completion_at(
        root.path(),
        once.definition.id,
        now + chrono::Duration::minutes(3),
    )
    .await
    .unwrap();
    let after_late_completion = get_at(root.path(), &actor(), once.definition.id, now)
        .await
        .unwrap();
    assert_eq!(
        after_late_completion.definition.status,
        AutomationStatus::Disabled
    );
    assert_eq!(
        after_late_completion.definition.anchor_at,
        disabled.definition.anchor_at
    );

    delete_at(root.path(), &actor(), once.definition.id)
        .await
        .unwrap();
    record_completion_at(root.path(), once.definition.id, Utc::now())
        .await
        .unwrap();
    assert!(super::store::read_all_at(root.path())
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn completed_automation_can_receive_a_disabled_replacement_schedule() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 10, 10, 0, 0).unwrap();
    let once = create_at(
        root.path(),
        &actor(),
        input(AutomationSchedule::Once {
            local_datetime: now.naive_utc() + chrono::Duration::hours(1),
            timezone: chrono_tz::UTC,
        }),
        now,
    )
    .await
    .unwrap();
    record_completion_at(root.path(), once.definition.id, now)
        .await
        .unwrap();

    let updated = update_at(
        root.path(),
        &actor(),
        once.definition.id,
        UpdateAutomation {
            schedule: Some(AutomationSchedule::Once {
                local_datetime: now.naive_utc() + chrono::Duration::hours(2),
                timezone: chrono_tz::UTC,
            }),
            status: Some(AutomationStatus::Disabled),
            ..Default::default()
        },
        now,
    )
    .await
    .unwrap();

    assert_eq!(updated.definition.status, AutomationStatus::Disabled);
}

#[tokio::test]
async fn once_in_the_past_is_rejected_on_create_and_update() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 10, 10, 0, 0).unwrap();
    let past = AutomationSchedule::Once {
        local_datetime: now.naive_utc() - chrono::Duration::minutes(1),
        timezone: chrono_tz::UTC,
    };
    assert_eq!(
        create_at(root.path(), &actor(), input(past.clone()), now)
            .await
            .unwrap_err(),
        AutomationError::InvalidSchedule
    );

    let created = create_at(
        root.path(),
        &actor(),
        input(AutomationSchedule::AfterCompletion { delay_minutes: 10 }),
        now,
    )
    .await
    .unwrap();
    assert_eq!(
        update_at(
            root.path(),
            &actor(),
            created.definition.id,
            UpdateAutomation {
                schedule: Some(past),
                ..Default::default()
            },
            now,
        )
        .await
        .unwrap_err(),
        AutomationError::InvalidSchedule
    );
}

#[tokio::test]
async fn identical_after_completion_schedule_does_not_move_its_anchor() {
    let root = tempfile::tempdir().unwrap();
    let now = Utc.with_ymd_and_hms(2026, 9, 10, 10, 0, 0).unwrap();
    let created = create_at(
        root.path(),
        &actor(),
        input(AutomationSchedule::AfterCompletion { delay_minutes: 10 }),
        now,
    )
    .await
    .unwrap();

    let updated = update_at(
        root.path(),
        &actor(),
        created.definition.id,
        UpdateAutomation {
            schedule: Some(AutomationSchedule::AfterCompletion { delay_minutes: 10 }),
            ..Default::default()
        },
        now + chrono::Duration::minutes(3),
    )
    .await
    .unwrap();

    assert_eq!(updated.definition.anchor_at, Some(now));
}
