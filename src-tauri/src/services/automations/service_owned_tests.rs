use super::*;
use crate::models::{AutomationDefinition, AutomationSchedule, AutomationStatus, AutomationTarget};
use chrono::{TimeZone, Utc};
use uuid::Uuid;

fn actor(origin: AutomationOrigin) -> AutomationActor {
    AutomationActor {
        origin,
        session_or_channel_id: "owner.test".into(),
        current_automation_id: None,
    }
}

fn owner() -> ExtensionActorIdentity {
    ExtensionActorIdentity {
        id: "owner.test".into(),
        version: "1.0.0".into(),
        fingerprint: "ab".repeat(32),
    }
}

fn definition(id: Uuid, schedule: AutomationSchedule) -> AutomationDefinition {
    AutomationDefinition {
        id,
        revision: 1,
        name: "Owned".into(),
        description: None,
        prompt: "Run".into(),
        creator_session_id: None,
        target: AutomationTarget::NewSession { project_id: None },
        provider: "codex-oauth".into(),
        model: "gpt-5.6-luna".into(),
        schedule,
        status: AutomationStatus::Disabled,
        created_at: Utc.with_ymd_and_hms(2026, 9, 1, 8, 0, 0).unwrap(),
        anchor_at: None,
        extension_owner: Some(super::ownership::new_owner(&owner())),
    }
}

async fn seed(root: &std::path::Path, value: AutomationDefinition) {
    super::store::mutate_at(root, |items| {
        items.push(value);
        Ok(())
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn native_edit_cannot_erase_extension_owner() {
    let root = tempfile::tempdir().unwrap();
    let id = Uuid::new_v4();
    let value = definition(id, cron());
    let expected = value.extension_owner.clone();
    seed(root.path(), value).await;
    super::service_mutations::update_at(
        root.path(),
        &actor(AutomationOrigin::UserInterface),
        id,
        UpdateAutomation {
            name: Some("Renamed".into()),
            ..Default::default()
        },
        Utc::now(),
    )
    .await
    .unwrap();
    assert_eq!(
        super::store::read_all_at(root.path()).await.unwrap()[0].extension_owner,
        expected
    );
}

#[tokio::test]
async fn automation_consent_is_bound_to_content_fingerprint() {
    let root = tempfile::tempdir().unwrap();
    let id = Uuid::new_v4();
    seed(root.path(), definition(id, cron())).await;
    let active =
        super::service_owned::set_active_at(root.path(), &owner(), id, 1, true, false, Utc::now())
            .await
            .unwrap();
    assert!(super::ownership::consent_is_current(&active.definition));
    let updated = super::service_owned::update_at(
        root.path(),
        &owner(),
        id,
        active.definition.revision,
        UpdateAutomation {
            prompt: Some("Changed".into()),
            ..Default::default()
        },
        Utc::now(),
    )
    .await
    .unwrap();
    assert_eq!(updated.definition.status, AutomationStatus::Disabled);
    assert!(!super::ownership::consent_is_current(&updated.definition));
}

#[tokio::test]
async fn after_completion_revision_does_not_revoke_unchanged_consent() {
    let root = tempfile::tempdir().unwrap();
    let id = Uuid::new_v4();
    let mut value = definition(id, AutomationSchedule::AfterCompletion { delay_minutes: 5 });
    value.anchor_at = Some(Utc::now());
    seed(root.path(), value).await;
    let active =
        super::service_owned::set_active_at(root.path(), &owner(), id, 1, true, false, Utc::now())
            .await
            .unwrap();
    super::service_mutations::record_completion_at(root.path(), id, Utc::now())
        .await
        .unwrap();
    let current = super::store::read_all_at(root.path())
        .await
        .unwrap()
        .remove(0);
    assert!(current.revision > active.definition.revision);
    assert!(super::ownership::consent_is_current(&current));
}

#[tokio::test]
async fn extension_activation_respects_global_pause() {
    let root = tempfile::tempdir().unwrap();
    let id = Uuid::new_v4();
    seed(root.path(), definition(id, cron())).await;
    assert_eq!(
        super::service_owned::set_active_at(root.path(), &owner(), id, 1, true, true, Utc::now(),)
            .await,
        Err(AutomationError::GloballyPaused)
    );
}

#[tokio::test]
async fn revoking_owner_disables_automations_and_requires_new_consent() {
    let root = tempfile::tempdir().unwrap();
    let id = Uuid::new_v4();
    seed(root.path(), definition(id, cron())).await;
    super::service_owned::set_active_at(root.path(), &owner(), id, 1, true, false, Utc::now())
        .await
        .unwrap();

    assert_eq!(
        super::service_owned::revoke_owner_at(root.path(), &owner().id)
            .await
            .unwrap(),
        1
    );
    let current = super::store::read_all_at(root.path())
        .await
        .unwrap()
        .remove(0);
    assert_eq!(current.status, AutomationStatus::Disabled);
    assert!(!super::ownership::consent_is_current(&current));
}

#[test]
fn summaries_expose_extension_origin_and_safe_inactive_reasons() {
    let now = Utc::now();
    let native = {
        let mut value = definition(Uuid::new_v4(), cron());
        value.extension_owner = None;
        value
    };
    let native_summary =
        super::service_helpers::summary_with_state(native, now, false, false, None);
    assert_eq!(native_summary.origin, AutomationOrigin::UserInterface);
    assert_eq!(native_summary.inactive_reason, None);

    let mut session_created = native_summary_definition(Uuid::new_v4());
    session_created.creator_session_id = Some("session-a".into());
    assert_eq!(
        super::service_helpers::summary_with_state(session_created, now, false, false, None).origin,
        AutomationOrigin::Session
    );

    let channel_created = super::service_helpers::new_definition(
        &AutomationActor {
            origin: AutomationOrigin::ExternalChannel,
            session_or_channel_id: "gateway:0123".into(),
            current_automation_id: None,
        },
        input("Channel"),
        now,
    );
    assert_eq!(
        super::service_helpers::summary_with_state(channel_created, now, false, false, None).origin,
        AutomationOrigin::ExternalChannel
    );

    let unavailable = super::service_helpers::summary_with_state(
        definition(Uuid::new_v4(), cron()),
        now,
        false,
        false,
        None,
    );
    assert_eq!(unavailable.origin, AutomationOrigin::Extension);
    assert_eq!(
        unavailable.inactive_reason,
        Some(AutomationInactiveReason::OwnerUnavailable)
    );

    let mut invalid = definition(Uuid::new_v4(), cron());
    invalid.extension_owner = Some(crate::models::AutomationExtensionOwnership::Invalid(
        serde_json::json!({"extensionId": "broken"}),
    ));
    let invalid = super::service_helpers::summary_with_state(invalid, now, false, false, None);
    assert_eq!(
        invalid.inactive_reason,
        Some(AutomationInactiveReason::OwnerInvalid)
    );
}

fn native_summary_definition(id: Uuid) -> AutomationDefinition {
    let mut value = definition(id, cron());
    value.extension_owner = None;
    value
}

#[tokio::test]
async fn concurrent_creates_share_global_limit() {
    let root = tempfile::tempdir().unwrap();
    super::store::mutate_at(root.path(), |items| {
        items.extend((0..63).map(|_| {
            let mut value = definition(Uuid::new_v4(), cron());
            value.extension_owner = None;
            value
        }));
        Ok(())
    })
    .await
    .unwrap();
    let first_owner = owner();
    let second_owner = ExtensionActorIdentity {
        id: "owner.neighbor".into(),
        version: "1.0.0".into(),
        fingerprint: "cd".repeat(32),
    };
    let first_actor = actor(AutomationOrigin::Extension);
    let second_actor = actor(AutomationOrigin::Extension);
    let first = super::service_owned::create_at(
        root.path(),
        &first_actor,
        &first_owner,
        input("first"),
        Utc::now(),
    );
    let second = super::service_owned::create_at(
        root.path(),
        &second_actor,
        &second_owner,
        input("second"),
        Utc::now(),
    );
    let (first, second) = tokio::join!(first, second);
    assert_ne!(first.is_ok(), second.is_ok());
    assert_eq!(
        super::store::read_all_at(root.path()).await.unwrap().len(),
        64
    );
}

#[tokio::test]
async fn previous_extension_versions_still_count_toward_owner_limit() {
    let root = tempfile::tempdir().unwrap();
    super::store::mutate_at(root.path(), |items| {
        for index in 0..super::types::MAX_AUTOMATIONS_PER_EXTENSION {
            let mut value = definition(Uuid::new_v4(), cron());
            if let Some(crate::models::AutomationExtensionOwnership::Valid(owner)) =
                value.extension_owner.as_mut()
            {
                owner.extension_version = format!("0.{index}.0");
                owner.extension_fingerprint = format!("{index:064x}");
            }
            items.push(value);
        }
        Ok(())
    })
    .await
    .unwrap();

    assert_eq!(
        super::service_owned::create_at(
            root.path(),
            &actor(AutomationOrigin::Extension),
            &owner(),
            input("overflow"),
            Utc::now(),
        )
        .await,
        Err(AutomationError::CapacityReached)
    );
}

fn input(name: &str) -> CreateAutomation {
    CreateAutomation {
        name: name.into(),
        description: None,
        prompt: "Run".into(),
        target: AutomationTarget::NewSession { project_id: None },
        provider: "codex-oauth".into(),
        model: "gpt-5.6-luna".into(),
        schedule: cron(),
        status: AutomationStatus::Disabled,
    }
}

fn cron() -> AutomationSchedule {
    AutomationSchedule::Cron {
        expression: "0 8 * * *".into(),
        timezone: chrono_tz::UTC,
    }
}
