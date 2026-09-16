use super::tool_interception::InterceptorRegistration;
use super::types::{ExtensionDiagnostic, ExtensionKind, MAX_RUNTIME_DIAGNOSTICS};

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
    runtime.mark_interceptor_disabled(&entry.extension_id, code);
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
    fn mark_interceptor_disabled(&self, extension_id: &str, code: &'static str) {
        self.record_interceptor_diagnostic(extension_id, code);
        if let Ok(mut status) = self.status.write() {
            status.active_extensions = status.active_extensions.saturating_sub(1);
        }
    }

    pub(super) fn record_interceptor_diagnostic(&self, extension_id: &str, code: &'static str) {
        let Ok(mut status) = self.status.write() else {
            return;
        };
        if status.diagnostics.len() >= MAX_RUNTIME_DIAGNOSTICS {
            status.diagnostics.remove(0);
        }
        status.diagnostics.push(ExtensionDiagnostic {
            extension_id: extension_id.to_string(),
            stage: super::types::HOST_LOAD_STAGE_REGISTER.to_string(),
            code: code.to_string(),
            occurred_at: super::diagnostic_time::now(),
            file: None,
            line: None,
            column: None,
        });
    }
}
