use super::core_bridge::ExtensionBridgeError;
use super::core_scope::{AgentCoreScope, CoreScopeRegistry};
use super::host_identity::HostIdentity;
use super::types::{ExtensionApiLevel, ExtensionEffect};
use crate::services::llm::request_purpose::RequestPurpose;
use crate::services::agent_local::subagent_tool_profile::SubagentToolProfile;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

pub(super) fn scoped_context(
    plan_active: bool,
    cancel: CancellationToken,
) -> super::call_context::ExtensionCallContext {
    scoped_context_for(plan_active, cancel, "manual", None, RequestPurpose::ManualChat)
}

fn scoped_context_for(
    plan_active: bool,
    cancel: CancellationToken,
    permission_mode: &str,
    profile: Option<SubagentToolProfile>,
    purpose: RequestPurpose,
) -> super::call_context::ExtensionCallContext {
    let registry = CoreScopeRegistry::default();
    let lease = registry
        .admit(
            HostIdentity::Official,
            1,
            AgentCoreScope {
                session_id: "session".into(),
                request_id: "request".into(),
                working_directory: PathBuf::from("."),
                permission_mode: permission_mode.into(),
                profile,
                purpose,
                cancel,
                on_event: crate::services::agent_local::stream_events::AgentEventEmitter::test(
                    "session".into(),
                ),
                plan_active,
            },
            "beaver.office.documents.create".into(),
            ExtensionEffect::ReadOnly,
            Instant::now() + Duration::from_secs(2),
        )
        .unwrap();
    let envelope = serde_json::to_value(lease.envelope()).unwrap();
    let scope = registry
        .resolve(
            envelope["id"].as_str().unwrap(),
            envelope["secret"].as_str().unwrap(),
            &HostIdentity::Official,
            1,
        )
        .unwrap();
    drop(lease);
    super::call_context::ExtensionCallContext::for_test_with_capabilities(
        HostIdentity::Official,
        ExtensionApiLevel::Advanced,
        vec!["memory".into(), "models".into(), "automations".into()],
    )
    .with_core_scope(scope)
}

#[tokio::test]
async fn automation_activation_always_requires_manual_confirmation() {
    let context = scoped_context_for(
        false,
        CancellationToken::new(),
        "auto",
        None,
        RequestPurpose::Automation,
    );
    let policy = super::core_api_dispatch::policy(&context, "automations.setActive").unwrap();
    assert_eq!(
        super::core_api_permissions::authorize(
            &context,
            "automations.setActive",
            &serde_json::json!({"active": true}),
            policy.effect,
        )
        .await,
        Err(ExtensionBridgeError::Denied)
    );
}

#[tokio::test]
async fn models_generate_is_denied_in_plan_explorer_and_chat() {
    for context in [
        scoped_context_for(true, CancellationToken::new(), "manual", None, RequestPurpose::ManualChat),
        scoped_context_for(
            false,
            CancellationToken::new(),
            "manual",
            Some(SubagentToolProfile::Explorer),
            RequestPurpose::ManualChat,
        ),
        scoped_context_for(false, CancellationToken::new(), "chat", None, RequestPurpose::ManualChat),
    ] {
        let policy = super::core_api_dispatch::policy(&context, "models.generate").unwrap();
        assert_eq!(
            super::core_api_permissions::authorize(&context, "models.generate", &serde_json::json!({}), policy.effect).await,
            Err(ExtensionBridgeError::Denied),
        );
    }
    assert!(!crate::services::llm::stream_dispatch::model_route_descriptor("xai-oauth")
        .unwrap()
        .generation_supported);
}

#[tokio::test]
async fn nested_call_cannot_upgrade_plan_or_parent_permissions() {
    let context = scoped_context(true, CancellationToken::new());
    let policy = super::core_api_dispatch::policy(&context, "memory.write").unwrap();
    assert_eq!(
        super::core_api_permissions::authorize(&context, "memory.write", &serde_json::json!({}), policy.effect).await,
        Err(ExtensionBridgeError::Denied)
    );
}

#[tokio::test]
async fn legacy_call_and_typed_call_share_authorization() {
    let context = scoped_context(true, CancellationToken::new());
    for method in ["mcp.tool.call", "memory.write"] {
        let policy = super::core_api_dispatch::policy(&context, method).unwrap();
        assert_eq!(
            super::core_api_permissions::authorize(&context, method, &serde_json::json!({}), policy.effect).await,
            Err(ExtensionBridgeError::Denied),
            "{method}"
        );
    }
}

#[tokio::test]
async fn stop_revokes_nested_work() {
    let cancel = CancellationToken::new();
    let context = scoped_context(false, cancel.clone());
    cancel.cancel();
    assert_eq!(
        super::core_api_permissions::authorize(
            &context,
            "mcp.tool.call",
            &serde_json::json!({}),
            ExtensionEffect::ExternalWrite
        )
        .await,
        Err(ExtensionBridgeError::Revoked)
    );
}

#[test]
fn contextual_method_without_scope_is_rejected() {
    let context = super::call_context::ExtensionCallContext::for_test_with_capabilities(
        HostIdentity::Official,
        ExtensionApiLevel::Advanced,
        vec!["memory".into()],
    );
    assert!(matches!(
        super::core_api_dispatch::policy(&context, "memory.read"),
        Err(ExtensionBridgeError::Context("core_context_required"))
    ));
}

#[test]
fn malformed_or_expired_scope_never_falls_back_to_ambient_legacy_access() {
    let context = super::call_context::ExtensionCallContext::for_test(
        HostIdentity::Official,
        ExtensionApiLevel::Advanced,
    )
    .with_core_scope_error("core_context_expired");

    for method in ["mcp.tool.call", "secrets.provider.get", "app.info"] {
        assert!(matches!(
            super::core_api_dispatch::policy(&context, method),
            Err(ExtensionBridgeError::Context("core_context_expired"))
        ));
    }
}
