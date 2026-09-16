use crate::models::{
    AutomationDefinition, AutomationExtensionOwner, AutomationExtensionOwnership,
    AutomationSchedule, AutomationStatus, AutomationTarget,
};
use chrono::Utc;
use uuid::Uuid;

fn definition() -> AutomationDefinition {
    AutomationDefinition {
        id: Uuid::new_v4(),
        revision: 1,
        name: "Owned".into(),
        description: None,
        prompt: "Run".into(),
        creator_session_id: None,
        target: AutomationTarget::NewSession { project_id: None },
        provider: "openai".into(),
        model: "model".into(),
        schedule: AutomationSchedule::Cron {
            expression: "0 * * * *".into(),
            timezone: chrono_tz::UTC,
        },
        status: AutomationStatus::Active,
        created_at: Utc::now(),
        anchor_at: None,
        extension_owner: None,
    }
}

fn owned() -> AutomationDefinition {
    let mut definition = definition();
    definition.extension_owner = Some(AutomationExtensionOwnership::Valid(
        AutomationExtensionOwner {
            extension_id: "com.example.owner".into(),
            extension_version: "1.0.0".into(),
            extension_fingerprint: "ab".repeat(32),
            approved_content_sha256: Some(
                crate::services::automations::extension_content_fingerprint(&definition).unwrap(),
            ),
            approved_at: Some(Utc::now()),
        },
    ));
    definition
}

#[test]
fn updated_extension_does_not_adopt_previous_automations() {
    let definition = owned();
    assert_eq!(
        super::extension_admission::admitted_owner(&definition, |_| false),
        Err("extension_unavailable")
    );
    assert_eq!(
        super::extension_admission::admitted_owner(&definition, |_| true),
        Ok(Some("com.example.owner"))
    );
}

#[test]
fn malformed_or_unapproved_owner_is_never_admitted() {
    let mut definition = owned();
    definition.extension_owner = Some(AutomationExtensionOwnership::Invalid(
        serde_json::json!({"extensionId": "com.example.owner"}),
    ));
    assert!(super::extension_admission::admitted_owner(&definition, |_| true).is_err());

    let mut definition = owned();
    if let Some(AutomationExtensionOwnership::Valid(owner)) = definition.extension_owner.as_mut() {
        owner.approved_content_sha256 = None;
        owner.approved_at = None;
    }
    assert!(super::extension_admission::admitted_owner(&definition, |_| true).is_err());
}
