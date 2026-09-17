use super::core_scope::{AgentCoreScope, CoreScopeRegistry};
use super::host_identity::HostIdentity;
use super::types::{ExtensionEffect, MAX_CONTEXTS_PER_HOST_IDENTITY};
use crate::services::agent_local::stream_events::AgentEventEmitter;
use crate::services::llm::request_purpose::RequestPurpose;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

fn input(cancel: CancellationToken) -> AgentCoreScope {
    AgentCoreScope {
        session_id: "session-a".into(),
        request_id: "request-a".into(),
        working_directory: PathBuf::from("/workspace"),
        permission_mode: "manual".into(),
        profile: None,
        purpose: RequestPurpose::ManualChat,
        cancel,
        on_event: AgentEventEmitter::test("session-a".into()),
        plan_active: false,
    }
}

fn credentials(lease: &super::core_scope::CoreScopeLease) -> (String, String) {
    let value = serde_json::to_value(lease.envelope()).unwrap();
    (
        value["id"].as_str().unwrap().to_string(),
        value["secret"].as_str().unwrap().to_string(),
    )
}

#[test]
fn context_is_bound_to_host_turn_and_generation() {
    let registry = CoreScopeRegistry::default();
    let cancel = CancellationToken::new();
    let identity = HostIdentity::ThirdParty("one.extension".into());
    let lease = registry
        .admit(
            identity.clone(),
            7,
            input(cancel.clone()),
            "example.tool".to_string(),
            ExtensionEffect::ReadOnly,
            Instant::now() + Duration::from_secs(2),
        )
        .unwrap();
    let (id, secret) = credentials(&lease);

    let resolved = registry.resolve(&id, &secret, &identity, 7).unwrap();
    assert_eq!(resolved.agent.session_id, "session-a");
    assert_eq!(resolved.agent.request_id, "request-a");
    assert_eq!(resolved.tool_effect, ExtensionEffect::ReadOnly);
    assert_eq!(resolved.agent.permission_mode, "manual");
    assert!(registry.resolve(&id, &secret, &identity, 8).is_err());
    assert!(registry
        .resolve(
            &id,
            &secret,
            &HostIdentity::ThirdParty("neighbor.extension".into()),
            7,
        )
        .is_err());

    cancel.cancel();
    assert_eq!(
        registry.resolve(&id, &secret, &identity, 7).err().unwrap(),
        "core_context_revoked"
    );
    drop(lease);
    assert_eq!(
        registry.resolve(&id, &secret, &identity, 7).err().unwrap(),
        "core_context_expired"
    );
}

#[test]
fn owner_saturation_does_not_starve_a_neighbor() {
    let registry = CoreScopeRegistry::default();
    let crowded = HostIdentity::ThirdParty("crowded.extension".into());
    let neighbor = HostIdentity::ThirdParty("neighbor.extension".into());
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut leases = Vec::new();
    for _ in 0..MAX_CONTEXTS_PER_HOST_IDENTITY {
        leases.push(
            registry
                .admit(
                    crowded.clone(),
                    1,
                    input(CancellationToken::new()),
                    "example.tool".to_string(),
                    ExtensionEffect::ReadOnly,
                    deadline,
                )
                .unwrap(),
        );
    }
    assert!(registry
        .admit(
            crowded,
            1,
            input(CancellationToken::new()),
            "example.tool".to_string(),
            ExtensionEffect::ReadOnly,
            deadline,
        )
        .is_err());
    assert!(registry
        .admit(
            neighbor,
            1,
            input(CancellationToken::new()),
            "example.tool".to_string(),
            ExtensionEffect::ReadOnly,
            deadline,
        )
        .is_ok());
}

#[test]
fn malformed_or_expired_credentials_fail_closed() {
    let registry = CoreScopeRegistry::default();
    let identity = HostIdentity::Official;
    let lease = registry
        .admit(
            identity.clone(),
            2,
            input(CancellationToken::new()),
            "example.tool".to_string(),
            ExtensionEffect::Unknown,
            Instant::now() - Duration::from_millis(1),
        )
        .unwrap();
    let (id, secret) = credentials(&lease);
    assert_eq!(
        registry.resolve(&id, &secret, &identity, 2).err().unwrap(),
        "core_context_expired"
    );
    assert_eq!(
        registry
            .resolve(&id, "not-a-secret", &identity, 2)
            .err()
            .unwrap(),
        "core_context_invalid"
    );
}
