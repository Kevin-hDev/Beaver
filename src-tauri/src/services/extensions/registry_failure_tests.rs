use super::host_identity::HostIdentity;
use super::registry_failure::{apply_identity_failure, identity_error_code};
use super::types::{
    ExtensionApiLevel, ExtensionContributions, ExtensionKind, ExtensionManifest, ExtensionRecord,
    ExtensionStatus,
};

fn record(id: &str) -> ExtensionRecord {
    ExtensionRecord {
        manifest: ExtensionManifest {
            id: id.to_string(),
            name: id.to_string(),
            version: "1.0.0".to_string(),
            beaver_api: "1".to_string(),
            runtime: "node".to_string(),
            main: Some("index.mjs".to_string()),
            ui: None,
            access: "full".to_string(),
            api_level: ExtensionApiLevel::Stable,
            essential: false,
            author: None,
            homepage: None,
            description: None,
        },
        kind: ExtensionKind::Local,
        source: "test".to_string(),
        origin: None,
        enabled: true,
        trusted: true,
        fingerprint: None,
        trusted_at: None,
        show_in_chat: true,
        status: ExtensionStatus::Loading,
        last_error: None,
        last_activated_at: None,
        sensitive_access_granted: false,
        contributions: ExtensionContributions::default(),
    }
}

#[test]
fn unavailable_host_uses_the_canonical_last_error_code() {
    assert_eq!(identity_error_code(), super::error_codes::HOST_UNAVAILABLE);
    let mut records = vec![record("target")];
    let mut affected = Vec::new();

    apply_identity_failure(
        &mut records,
        &HostIdentity::ThirdParty("target".to_string()),
        super::error_codes::HOST_UNAVAILABLE,
        false,
        &mut affected,
    );

    assert_eq!(
        records[0].last_error.as_deref(),
        Some(super::error_codes::HOST_UNAVAILABLE)
    );
    assert!(super::error_codes::ALL.contains(&records[0].last_error.as_deref().unwrap()));
}

#[test]
fn unconfirmed_stop_keeps_only_the_target_disabled_and_in_error() {
    let mut target = record("target");
    target.enabled = false;
    let healthy = record("healthy");
    let mut records = vec![target, healthy];
    let mut affected = Vec::new();

    apply_identity_failure(
        &mut records,
        &HostIdentity::ThirdParty("target".to_string()),
        super::error_codes::STOP_UNCONFIRMED,
        true,
        &mut affected,
    );

    assert_eq!(affected, vec!["target"]);
    assert!(!records[0].enabled);
    assert_eq!(records[0].status, ExtensionStatus::Error);
    assert_eq!(
        records[0].last_error.as_deref(),
        Some(super::error_codes::STOP_UNCONFIRMED)
    );
    assert_eq!(records[1].status, ExtensionStatus::Loading);
    assert!(records[1].enabled);
}
