use super::tool_interception::InterceptorRegistration;
use super::types::ExtensionKind;

pub(super) async fn disable(entry: &InterceptorRegistration, code: &'static str) {
    let Ok(runtime) = super::runtime::global() else {
        return;
    };
    runtime.tool_interceptors.remove(&entry.extension_id);
    let _ = runtime
        .revoke_extension(
            &entry.identity,
            super::runtime_lifecycle::new_stop_deadline(),
        )
        .await;
    let persisted = super::registry_failure::disable_extension(&entry.extension_id, code).is_ok();
    crate::services::agent_local::permission_gate::clear_extension(&entry.extension_id).await;
    let _ = super::loading_marker::clear_if_matches(&entry.extension_id);
    let _ = super::loading_marker::ui_clear_if_matches(&entry.extension_id);
    runtime.mark_interceptor_disabled();
    if !persisted {
        runtime.set_state(
            super::types::HostState::Error,
            Some(super::error_codes::STORAGE_FAILED.to_string()),
            0,
        );
    } else if matches!(entry.identity, super::host_identity::HostIdentity::Official) {
        restart_official_neighbor().await;
    }
    runtime.hosts.lock().await.emit_changed();
}

async fn restart_official_neighbor() {
    let Ok(records) = super::registry::list() else {
        return;
    };
    let Some(id) = records
        .iter()
        .find(|record| record.kind == ExtensionKind::Builtin && record.enabled && record.trusted)
        .map(|record| record.manifest.id.as_str())
    else {
        return;
    };
    let _ =
        super::runtime_lifecycle::ensure_running(id, super::runtime_lifecycle::new_stop_deadline())
            .await;
}

impl super::runtime::ExtensionRuntime {
    fn mark_interceptor_disabled(&self) {
        if let Ok(mut status) = self.status.write() {
            status.active_extensions = status.active_extensions.saturating_sub(1);
        }
    }
}
