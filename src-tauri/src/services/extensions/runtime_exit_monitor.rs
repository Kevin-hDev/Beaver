use std::sync::Arc;

use super::runtime::ExtensionRuntime;

impl ExtensionRuntime {
    pub(super) fn start_exit_monitor(
        self: &Arc<Self>,
        mut receiver: tokio::sync::mpsc::Receiver<super::runtime_hosts::HostExitNotice>,
    ) -> Result<(), String> {
        let runtime = Arc::clone(self);
        self.work
            .spawn_lifecycle(move |cancel| async move {
                loop {
                    let notice = tokio::select! {
                        _ = cancel.cancelled() => break,
                        notice = receiver.recv() => match notice {
                            Some(notice) => notice,
                            None => break,
                        },
                    };
                    runtime.handle_host_exit(notice).await;
                }
            })
            .map_err(|error| error.public_code().to_string())
    }

    async fn handle_host_exit(&self, notice: super::runtime_hosts::HostExitNotice) {
        let kind = self.hosts.lock().await.exit_kind(&notice);
        if kind == Some(super::runtime_host_generation::HostExitKind::Requested) {
            self.reap_requested_exit(&notice).await;
            return;
        }
        if kind.is_none() {
            return;
        }
        self.set_state(
            super::types::HostState::Error,
            Some(super::error_codes::HOST_UNAVAILABLE.to_string()),
            0,
        );
        let ids = super::registry_sync::mark_identity_error(&notice.identity);
        for id in ids {
            crate::services::agent_local::permission_gate::clear_extension(&id).await;
        }
        self.hosts.lock().await.emit_changed();
        let _ = self
            .stop_host(
                &notice.identity,
                super::runtime_lifecycle::new_stop_deadline(),
            )
            .await;
    }

    async fn reap_requested_exit(&self, notice: &super::runtime_hosts::HostExitNotice) {
        let expected = self
            .hosts
            .lock()
            .await
            .snapshot(&notice.identity)
            .filter(|(_, generation, _)| *generation == notice.generation)
            .map(|(_, _, process)| process);
        if let Some(process) = expected {
            let _ = self
                .stop_host_if_current(
                    &notice.identity,
                    Some(&process),
                    super::runtime_lifecycle::new_stop_deadline(),
                    false,
                )
                .await;
        }
    }
}
