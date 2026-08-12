use super::service::{GatewayService, GATEWAY_CONTROL_STOP_TIMEOUT};
use super::service_state::build_health;
use super::types::{ChannelHealthEntry, ChannelKey, ChannelStatus};
use super::work_supervision::GatewayWorkServices;
use std::time::Instant;
use tauri::Emitter;

impl GatewayService {
    pub async fn stop(&self) -> bool {
        self.stop_and_wait(Instant::now() + GATEWAY_CONTROL_STOP_TIMEOUT)
            .await
    }

    pub async fn stop_and_wait(&self, deadline: Instant) -> bool {
        self.cancel_active_run();
        let Ok(mut current_run) =
            tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), self.run.lock())
                .await
        else {
            return false;
        };
        self.stop_locked(&mut current_run, deadline).await
    }

    pub(super) async fn stop_locked(
        &self,
        current_run: &mut Option<GatewayWorkServices>,
        deadline: Instant,
    ) -> bool {
        self.stop_locked_with_audit(current_run, deadline, audit_stopped)
            .await
    }

    async fn stop_locked_with_audit<Audit, AuditFuture>(
        &self,
        current_run: &mut Option<GatewayWorkServices>,
        deadline: Instant,
        audit: Audit,
    ) -> bool
    where
        Audit: FnOnce(Vec<ChannelKey>) -> AuditFuture,
        AuditFuture: std::future::Future<Output = bool>,
    {
        let keys = self.mark_stopping().await;
        if let Some(work) = current_run.as_ref() {
            work.begin_closing();
        }
        let audit_ok = tokio::time::timeout_at(
            tokio::time::Instant::from_std(deadline),
            audit(keys.clone()),
        )
        .await
        .unwrap_or(false);
        let stopped = match current_run.take() {
            Some(work) => work.stop_and_wait(deadline).await,
            None => true,
        };
        if stopped {
            self.publish_off(&keys, audit_ok).await;
        }
        stopped
    }

    #[cfg(test)]
    pub(super) async fn stop_and_wait_with_audit_for_test<Audit, AuditFuture>(
        &self,
        deadline: Instant,
        audit: Audit,
    ) -> bool
    where
        Audit: FnOnce(Vec<ChannelKey>) -> AuditFuture,
        AuditFuture: std::future::Future<Output = bool>,
    {
        self.cancel_active_run();
        let Ok(mut current_run) =
            tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), self.run.lock())
                .await
        else {
            return false;
        };
        self.stop_locked_with_audit(&mut current_run, deadline, audit)
            .await
    }

    fn cancel_active_run(&self) {
        self.active_cancel
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .cancel();
    }

    async fn mark_stopping(&self) -> Vec<ChannelKey> {
        let mut state = self.state.write().await;
        state.cancel.cancel();
        let keys = state.channels.keys().cloned().collect::<Vec<_>>();
        for entry in state.channels.values_mut() {
            entry.cancel.cancel();
            entry.status = ChannelStatus::Stopping;
        }
        state.adapters.clear();
        keys
    }

    async fn publish_off(&self, keys: &[ChannelKey], audit_ok: bool) {
        let mut state = self.state.write().await;
        for entry in state.channels.values_mut() {
            entry.status = ChannelStatus::Off;
            if !audit_ok {
                entry.error = Some("auditUnavailable".to_string());
            }
        }
        let app = self
            .app
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        if let Some(app) = app {
            let _ = app.emit("gateway-status-changed", build_health(&state));
            let error = (!audit_ok).then_some("auditUnavailable".to_string());
            for key in keys {
                let _ = app.emit(
                    "gateway-channel-status",
                    ChannelHealthEntry {
                        channel_id: key.channel_id.clone(),
                        account_id: key.account_id.clone(),
                        status: ChannelStatus::Off,
                        error: error.clone(),
                    },
                );
            }
        }
    }
}

async fn audit_stopped(keys: Vec<ChannelKey>) -> bool {
    let task = tokio::task::spawn_blocking(move || {
        keys.iter()
            .all(|key| super::service_audit::channel_stopped(key, None, None).is_ok())
    });
    matches!(task.await, Ok(true))
}
