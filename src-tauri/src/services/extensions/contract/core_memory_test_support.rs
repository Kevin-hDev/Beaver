use super::super::super::core_bridge::{CoreResponse, ExtensionBridgeError};
use super::super::super::core_scope::{AgentCoreScope, CoreScopeRegistry};
use super::super::super::host_identity::HostIdentity;
use super::super::super::types::{ExtensionApiLevel, ExtensionEffect};
use crate::services::agent_local::memory_paths::MemoryLayout;
use crate::services::llm::request_purpose::RequestPurpose;
use serde_json::Value;
use std::path::Path;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

pub(super) fn context(
    session: &str,
    working_directory: &Path,
) -> super::super::super::call_context::ExtensionCallContext {
    let registry = CoreScopeRegistry::default();
    let lease = registry
        .admit(
            HostIdentity::Official,
            1,
            AgentCoreScope {
                session_id: session.into(),
                request_id: uuid::Uuid::new_v4().to_string(),
                working_directory: working_directory.into(),
                permission_mode: "auto".into(),
                profile: None,
                purpose: RequestPurpose::ManualChat,
                cancel: CancellationToken::new(),
                on_event: crate::services::agent_local::stream_events::AgentEventEmitter::test(
                    session.into(),
                ),
                plan_active: false,
            },
            "fixture.memory".into(),
            ExtensionEffect::LocalWrite,
            Instant::now() + Duration::from_secs(5),
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
    super::super::super::call_context::ExtensionCallContext::for_test_with_capabilities(
        HostIdentity::Official,
        ExtensionApiLevel::Stable,
        vec!["memory".into()],
    )
    .with_core_scope(scope)
}

pub(super) fn topic(body: &str) -> String {
    format!(
        "---\nid: placeholder\nscope: global\ntype: preference\nstatus: stale\n\
         title: Interface compacte\nsummary: Préférence durable.\ncreated_at: old\n\
         updated_at: old\ntags: [ui]\nsource: parent\nsession_id: old\n---\n{body}"
    )
}

pub(super) async fn invoke(
    context: &super::super::super::call_context::ExtensionCallContext,
    method: &str,
    params: Value,
    layout: &MemoryLayout,
) -> Result<Value, ExtensionBridgeError> {
    match super::super::call_with_layout(context, method, &params, layout).await? {
        CoreResponse::Json(value) => Ok(value),
        CoreResponse::Secret(_) => panic!("memory never returns secrets"),
    }
}
