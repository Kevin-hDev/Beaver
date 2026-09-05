use super::*;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use zeroize::Zeroizing;

#[test]
fn authorization_snapshot_rejects_spoofed_disabled_and_mismatched_channels() {
    let mut record = super::super::builtin::records().unwrap().remove(0);
    record.kind = super::super::types::ExtensionKind::Local;
    record.manifest.id = "com.example.bound".to_string();
    record.enabled = true;
    record.trusted = true;
    let bound = super::super::call_context::ExtensionCallContext::for_test(
        super::super::host_identity::HostIdentity::ThirdParty(record.manifest.id.clone()),
        record.manifest.api_level.clone(),
    );
    assert!(super::super::registry_access::authorized_records(
        std::slice::from_ref(&record),
        &bound
    ));

    let spoofed = super::super::call_context::ExtensionCallContext::for_test(
        super::super::host_identity::HostIdentity::ThirdParty("com.example.other".to_string()),
        record.manifest.api_level.clone(),
    );
    assert!(!super::super::registry_access::authorized_records(
        std::slice::from_ref(&record),
        &spoofed
    ));
    record.enabled = false;
    assert!(!super::super::registry_access::authorized_records(
        &[record],
        &bound
    ));

    let mut untrusted = super::super::builtin::records().unwrap().remove(0);
    untrusted.kind = super::super::types::ExtensionKind::Local;
    untrusted.manifest.id = "com.example.bound".to_string();
    untrusted.enabled = true;
    untrusted.trusted = false;
    assert!(!super::super::registry_access::authorized_records(
        std::slice::from_ref(&untrusted),
        &bound
    ));
    untrusted.trusted = true;
    untrusted.manifest.api_level = super::super::types::ExtensionApiLevel::Advanced;
    assert!(!super::super::registry_access::authorized_records(
        &[untrusted],
        &bound
    ));
}

#[tokio::test]
async fn rejects_an_extension_identity_claimed_by_node_params() {
    let context = super::super::call_context::ExtensionCallContext::for_test(
        super::super::host_identity::HostIdentity::ThirdParty("com.example.bound".to_string()),
        super::super::types::ExtensionApiLevel::Stable,
    );
    let result = call(
        &context,
        "app.info",
        Some(&json!({"extensionId": "com.beaver.office.word"})),
    )
    .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn stable_channel_rejects_an_advanced_alias_not_declared_by_the_contract() {
    let context = super::super::call_context::ExtensionCallContext::for_test(
        super::super::host_identity::HostIdentity::ThirdParty("com.example.stable".to_string()),
        super::super::types::ExtensionApiLevel::Stable,
    );

    let result = call(&context, "unstable.app.info", Some(&json!({}))).await;

    assert!(result.is_err());
}

#[test]
fn method_policy_rejects_advanced_and_notification_entries_for_stable_requests() {
    use super::super::types::ExtensionApiLevel;

    assert!(method_is_allowed(
        &ExtensionApiLevel::Stable,
        "stable",
        "request"
    ));
    assert!(!method_is_allowed(
        &ExtensionApiLevel::Stable,
        "advanced",
        "request"
    ));
    assert!(method_is_allowed(
        &ExtensionApiLevel::Advanced,
        "advanced",
        "request"
    ));
    assert!(!method_is_allowed(
        &ExtensionApiLevel::Advanced,
        "stable",
        "notification"
    ));
}

#[test]
fn request_parameter_policy_rejects_a_node_claimed_identity() {
    assert!(validate_request_params(&json!({"providerId": "openai"})).is_ok());
    assert!(validate_request_params(&json!({
        "providerId": "openai",
        "extensionId": "com.example.spoofed"
    }))
    .is_err());
}

#[tokio::test]
async fn advanced_channel_still_rejects_a_method_name_absent_from_the_contract() {
    let context = super::super::call_context::ExtensionCallContext::for_test(
        super::super::host_identity::HostIdentity::ThirdParty("com.example.advanced".to_string()),
        super::super::types::ExtensionApiLevel::Advanced,
    );

    let result = call(&context, "unstable.app.info", Some(&json!({}))).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn revocation_interrupts_an_in_flight_core_operation() {
    let context = super::super::call_context::ExtensionCallContext::for_test(
        super::super::host_identity::HostIdentity::ThirdParty("com.example.waiting".to_string()),
        super::super::types::ExtensionApiLevel::Stable,
    );
    let revoked = context.revoked().clone();

    let call = tokio::spawn(async move {
        await_unrevoked(&context, std::time::Duration::from_secs(1), async {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            Ok(CoreResponse::Json(json!({"secret": "must-not-arrive"})))
        })
        .await
    });
    tokio::task::yield_now().await;
    revoked.cancel();

    assert!(matches!(
        call.await.unwrap(),
        Err(ExtensionBridgeError::Revoked)
    ));
}

#[tokio::test]
async fn timeout_stops_waiting_without_starting_the_operation_twice() {
    let context = super::super::call_context::ExtensionCallContext::for_test(
        super::super::host_identity::HostIdentity::ThirdParty("com.example.timeout".to_string()),
        super::super::types::ExtensionApiLevel::Stable,
    );
    let starts = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&starts);

    let result = await_unrevoked(&context, std::time::Duration::from_millis(10), async move {
        observed.fetch_add(1, Ordering::SeqCst);
        std::future::pending::<Result<CoreResponse, ()>>().await
    })
    .await;

    assert!(matches!(result, Err(ExtensionBridgeError::Timeout)));
    assert_eq!(starts.load(Ordering::SeqCst), 1);
}

#[test]
fn sensitive_resource_scope_never_falls_back_to_another_requested_name() {
    let allowed = vec!["TOKEN_FOR_CONNECTOR_A".to_string()];

    assert!(super::super::core_secrets::requested_resource(
        &allowed,
        "TOKEN_FOR_CONNECTOR_A"
    ));
    assert!(!super::super::core_secrets::requested_resource(
        &allowed,
        "TOKEN_FOR_CONNECTOR_B"
    ));
}

#[test]
fn successful_sensitive_access_marks_only_the_bound_record_in_memory() {
    let mut records = super::super::builtin::records().unwrap();
    records[0].kind = super::super::types::ExtensionKind::Local;
    records[0].manifest.id = "com.example.bound".to_string();
    records[0].enabled = true;
    records[0].trusted = true;

    assert!(super::super::registry_access::mark_sensitive_identity(
        &mut records,
        &super::super::host_identity::HostIdentity::ThirdParty("com.example.bound".to_string())
    ));
    assert!(records[0].sensitive_access_granted);
    assert!(records
        .iter()
        .skip(1)
        .all(|record| !record.sensitive_access_granted));
}

#[test]
fn an_already_marked_sensitive_identity_needs_no_registry_rewrite() {
    let mut records = super::super::builtin::records().unwrap();
    for record in &mut records {
        record.sensitive_access_granted = true;
    }

    assert!(super::super::registry_access::sensitive_identity_is_marked(
        &records,
        &super::super::host_identity::HostIdentity::Official,
    ));
}

#[test]
fn bridge_finalization_withholds_a_secret_when_the_real_journal_write_fails() {
    let temporary = tempfile::tempdir().unwrap();
    let context = super::super::call_context::ExtensionCallContext::for_test(
        super::super::host_identity::HostIdentity::ThirdParty("com.example.audit".to_string()),
        super::super::types::ExtensionApiLevel::Stable,
    );
    let outcome = Ok(CoreResponse::Secret(Zeroizing::new(
        "AUDIT-FAKE-KEY".to_string(),
    )));

    let result = finalize_response(
        outcome,
        || Ok(()),
        |access_result| {
            super::super::access_log::write_core_at(
                temporary.path(),
                &context,
                "secrets.provider.get",
                access_result,
            )
        },
    );

    assert!(matches!(result, Err(ExtensionBridgeError::Failed)));
}

#[test]
fn sensitive_marking_failure_is_audited_as_failed_and_withholds_the_secret() {
    let mut audited = None;
    let result = finalize_response(
        Ok(CoreResponse::Secret(Zeroizing::new(
            "AUDIT-FAKE-KEY".to_string(),
        ))),
        || Err("registry unavailable".to_string()),
        |access_result| {
            audited = Some(access_result);
            Ok(())
        },
    );

    assert!(matches!(result, Err(ExtensionBridgeError::Failed)));
    assert_eq!(
        audited,
        Some(super::super::access_log::AccessResult::Failed)
    );
}
