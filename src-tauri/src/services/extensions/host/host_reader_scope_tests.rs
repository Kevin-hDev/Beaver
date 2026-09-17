use super::core_scope::{AgentCoreScope, CoreScopeRegistry};
use super::host_identity::HostIdentity;
use super::types::{ExtensionApiLevel, ExtensionEffect};
use crate::services::llm::request_purpose::RequestPurpose;
use serde_json::json;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

#[test]
fn private_scope_is_removed_and_bound_before_core_routing() {
    let registry = CoreScopeRegistry::default();
    let lease = registry
        .admit(
            HostIdentity::Official,
            4,
            AgentCoreScope {
                session_id: "session".into(),
                request_id: "request".into(),
                working_directory: PathBuf::from("."),
                permission_mode: "auto".into(),
                profile: None,
                purpose: RequestPurpose::ManualChat,
                cancel: CancellationToken::new(),
                on_event: crate::services::agent_local::stream_events::AgentEventEmitter::test(
                    "session".into(),
                ),
                plan_active: false,
            },
            "beaver.office.documents.create".into(),
            ExtensionEffect::ReadOnly,
            Instant::now() + Duration::from_secs(2),
        )
        .unwrap();
    let mut params = Some(json!({
        "value": "author-visible",
        super::types::CORE_CONTEXT_ENVELOPE_FIELD: lease.envelope(),
    }));
    let context = super::host_reader_scope::attach(
        super::call_context::ExtensionCallContext::for_test(
            HostIdentity::Official,
            ExtensionApiLevel::Advanced,
        ),
        &mut params,
        &registry,
        &HostIdentity::Official,
        4,
    );
    assert!(context.core_scope().is_some());
    assert_eq!(params.unwrap(), json!({"value": "author-visible"}));
}

#[test]
fn official_group_has_one_generation_authority_without_plugin_isolation_claim() {
    let mut params = Some(json!({
        super::types::CORE_CONTEXT_ENVELOPE_FIELD: {
            "id": uuid::Uuid::new_v4(),
            "secret": "00".repeat(32),
            "remainingMs": 10,
        }
    }));
    let context = super::host_reader_scope::attach(
        super::call_context::ExtensionCallContext::for_test(
            HostIdentity::Official,
            ExtensionApiLevel::Advanced,
        ),
        &mut params,
        &CoreScopeRegistry::default(),
        &HostIdentity::Official,
        1,
    );
    assert_eq!(context.core_scope_error(), Some("core_context_expired"));
}
