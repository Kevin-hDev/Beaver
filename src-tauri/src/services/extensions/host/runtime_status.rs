use super::runtime::ExtensionRuntime;
use super::types::{ExtensionDiagnostic, HostState};

impl ExtensionRuntime {
    pub(super) fn set_running(&self, active: usize, diagnostics: Vec<ExtensionDiagnostic>) {
        if let Ok(mut status) = self.status.write() {
            status.state = HostState::Running;
            status.active_extensions = active;
            status.last_error = None;
            status.diagnostics = diagnostics;
        }
    }

    pub(super) fn set_host_version(&self, hello: &super::protocol::HelloResult) {
        if let Ok(mut status) = self.status.write() {
            status.node_version = Some(hello.node_version.clone());
            status.jiti_version = hello.jiti_version.clone();
            status.api_version = hello.api_version.clone();
        }
    }

    pub(super) fn set_state(&self, state: HostState, error: Option<String>, active: usize) {
        if let Ok(mut status) = self.status.write() {
            status.state = state.clone();
            status.active_extensions = active;
            status.last_error = error;
            if state != HostState::Running {
                status.diagnostics.clear();
            }
        }
    }
}
