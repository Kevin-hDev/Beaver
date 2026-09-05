use super::host_identity::HostIdentity;
use super::host_process::HostProcess;
use super::runtime::ExtensionRuntime;
use super::runtime_hosts::HostStartReason;
use super::types::ExtensionApiLevel;
use std::sync::Arc;
use std::time::Instant;

impl ExtensionRuntime {
    pub(super) async fn ensure_channel(
        &self,
        identity: HostIdentity,
        api_level: ExtensionApiLevel,
        start_reason: HostStartReason,
    ) -> Result<Arc<HostProcess>, String> {
        if let Some(((current_level, _, process), usable)) = self.current_channel(&identity).await {
            if usable && current_level == api_level && process.is_alive() {
                return Ok(process);
            }
            if self
                .stop_host_if_current(
                    &identity,
                    Some(&process),
                    super::runtime_lifecycle::new_stop_deadline(),
                    true,
                )
                .await
                == super::runtime::StopHostOutcome::Unconfirmed
            {
                return Err(super::error_codes::HOST_UNAVAILABLE.to_string());
            }
        }
        let reservation = {
            let mut hosts = self.hosts.lock().await;
            if !hosts.admit_spawn(&identity, start_reason) {
                return Err(super::error_codes::HOST_UNAVAILABLE.to_string());
            }
            hosts.reserve(identity.clone())?
        };
        let paths = self
            .paths
            .as_ref()
            .ok_or_else(|| super::error_codes::HOST_UNAVAILABLE.to_string())?;
        let process = Arc::new(
            HostProcess::spawn_bound(
                paths,
                &self.work,
                reservation.spawn_binding(),
                super::runtime_lifecycle::new_stop_deadline(),
                reservation.temporary_directory(),
            )
            .await?,
        );
        let Ok(hello) = super::runtime_host_load::validate_hello(&process).await else {
            return Err(self
                .reject_spawn(
                    identity,
                    reservation,
                    api_level,
                    process,
                    super::runtime_lifecycle::new_stop_deadline(),
                )
                .await);
        };
        self.set_host_version(&hello);
        let bind =
            self.hosts
                .lock()
                .await
                .bind(reservation, api_level.clone(), Arc::clone(&process));
        if let Err(reservation) = bind {
            return Err(self
                .reject_spawn(
                    identity,
                    reservation,
                    api_level,
                    process,
                    super::runtime_lifecycle::new_stop_deadline(),
                )
                .await);
        }
        Ok(process)
    }

    async fn current_channel(
        &self,
        identity: &HostIdentity,
    ) -> Option<((ExtensionApiLevel, u64, Arc<HostProcess>), bool)> {
        let hosts = self.hosts.lock().await;
        hosts
            .snapshot(identity)
            .map(|snapshot| (snapshot, hosts.usable_snapshot(identity).is_some()))
    }

    async fn reject_spawn(
        &self,
        identity: HostIdentity,
        reservation: super::runtime_hosts::HostReservation,
        api_level: ExtensionApiLevel,
        process: Arc<HostProcess>,
        deadline: Instant,
    ) -> String {
        super::runtime_failed_spawn::terminate_or_retain(
            self,
            &identity,
            reservation,
            api_level,
            process,
            deadline,
        )
        .await
    }
}
