use super::core_bridge::ExtensionBridgeError;
use crate::models::{
    AutomationDefinition, AutomationExtensionOwner, AutomationExtensionOwnership,
    AutomationSchedule, AutomationStatus, AutomationTarget,
};
use chrono::{TimeZone, Utc};
use serde_json::json;
use uuid::Uuid;

#[test]
fn public_automation_hides_execution_and_owner_fields() {
    let value = AutomationDefinition {
        id: Uuid::nil(),
        revision: 3,
        name: "Owned".into(),
        description: None,
        prompt: "Run".into(),
        creator_session_id: None,
        target: AutomationTarget::NewSession { project_id: Some("project".into()) },
        provider: "provider".into(),
        model: "model".into(),
        schedule: AutomationSchedule::AfterCompletion { delay_minutes: 5 },
        status: AutomationStatus::Disabled,
        created_at: Utc.with_ymd_and_hms(2026, 9, 1, 8, 0, 0).unwrap(),
        anchor_at: None,
        extension_owner: Some(AutomationExtensionOwnership::Valid(AutomationExtensionOwner {
            extension_id: "owner.test".into(),
            extension_version: "1.0.0".into(),
            extension_fingerprint: "ab".repeat(32),
            approved_content_sha256: None,
            approved_at: None,
        })),
    };
    let public = super::core_automations_params::public_automation(value);
    assert_eq!(public["name"], "Owned");
    assert_eq!(public["active"], false);
    for hidden in ["target", "provider", "model", "creatorSessionId", "extensionOwner"] {
        assert!(public.get(hidden).is_none(), "{hidden}");
    }
}

#[test]
fn public_schedule_is_strict_and_cursor_is_bounded_to_numbers() {
    assert_eq!(
        super::core_automations_params::schedule(Some(&json!({
            "kind": "after_completion",
            "delay_minutes": 5,
            "extra": true
        }))),
        Err(ExtensionBridgeError::Denied)
    );
    assert_eq!(
        super::core_automations_params::cursor(&json!({"cursor": "not-a-number"})),
        Err(ExtensionBridgeError::Denied)
    );
}

#[test]
fn automation_and_subagent_contexts_cannot_schedule_descendants() {
    for (automation, subagent) in [(true, false), (false, true)] {
        assert!(super::core_automations::restricted_execution(
            "automations.create",
            &serde_json::json!({}),
            automation,
            subagent,
        ));
        assert!(super::core_automations::restricted_execution(
            "automations.setActive",
            &serde_json::json!({"active": true}),
            automation,
            subagent,
        ));
        assert!(!super::core_automations::restricted_execution(
            "automations.setActive",
            &serde_json::json!({"active": false}),
            automation,
            subagent,
        ));
        assert!(!super::core_automations::restricted_execution(
            "automations.list",
            &serde_json::json!({}),
            automation,
            subagent,
        ));
    }
}

#[test]
fn activation_confirmation_contains_every_approved_automation_field() {
    let definition = AutomationDefinition {
        id: Uuid::nil(),
        revision: 1,
        name: "Nightly report".into(),
        description: None,
        prompt: "Summarize the project".into(),
        creator_session_id: None,
        target: AutomationTarget::NewSession { project_id: Some("project".into()) },
        provider: "openai".into(),
        model: "gpt".into(),
        schedule: AutomationSchedule::AfterCompletion { delay_minutes: 5 },
        status: AutomationStatus::Disabled,
        created_at: Utc::now(),
        anchor_at: None,
        extension_owner: None,
    };
    let summary = super::core_automations::approval_arguments_from_definition(definition);

    assert_eq!(summary["name"], "Nightly report");
    assert_eq!(summary["prompt"], "Summarize the project");
    assert_eq!(summary["provider"], "openai");
    assert_eq!(summary["model"], "gpt");
    assert_eq!(summary["target"]["project_id"], "project");
    assert_eq!(summary["schedule"]["kind"], "after_completion");
    assert_eq!(summary["futureAgentExecution"], true);
}
