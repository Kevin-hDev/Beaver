use super::host_identity::HostIdentity;
use super::host_process::HostProcess;
use super::runtime::ExtensionRuntime;
use super::runtime_hosts::HostReservation;
use super::types::ExtensionApiLevel;
use std::sync::Arc;
use std::time::Instant;

pub(super) async fn terminate_or_retain(
    runtime: &ExtensionRuntime,
    identity: &HostIdentity,
    reservation: HostReservation,
    api_level: ExtensionApiLevel,
    process: Arc<HostProcess>,
    deadline: Instant,
) -> String {
    runtime.hosts.lock().await.revoke_reservation(&reservation);
    if process.kill(deadline).await {
        return super::error_codes::HOST_UNAVAILABLE.to_string();
    }
    runtime
        .hosts
        .lock()
        .await
        .retain_failed(reservation, api_level, process);
    runtime.mark_stop_unconfirmed(identity).await;
    super::error_codes::HOST_UNAVAILABLE.to_string()
}
