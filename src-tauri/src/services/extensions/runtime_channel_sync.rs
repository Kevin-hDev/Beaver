use super::host_identity::HostIdentity;
use super::runtime::ExtensionRuntime;
use super::runtime_hosts::HostStartReason;
use std::collections::BTreeSet;
use std::time::Instant;

impl ExtensionRuntime {
    pub(super) async fn sync_hosts(&self, deadline: Instant) -> Result<bool, String> {
        self.sync_hosts_with_recovery(deadline, None, HostStartReason::InitialOrManual)
            .await
    }

    pub(super) async fn sync_hosts_automatically(&self, deadline: Instant) -> Result<bool, String> {
        self.sync_hosts_with_recovery(deadline, None, HostStartReason::Automatic)
            .await
    }

    pub(super) async fn retry_host_load(
        &self,
        extension_id: String,
        attempts: u8,
        deadline: Instant,
    ) -> Result<bool, String> {
        self.sync_hosts_with_recovery(
            deadline,
            Some(super::runtime_sync::RecoveryPreflight::Retry(
                extension_id,
                attempts,
            )),
            HostStartReason::InitialOrManual,
        )
        .await
    }

    async fn sync_hosts_with_recovery(
        &self,
        deadline: Instant,
        forced_recovery: Option<super::runtime_sync::RecoveryPreflight>,
        start_reason: HostStartReason,
    ) -> Result<bool, String> {
        let _sync = self.sync.lock().await;
        let paths = self
            .paths
            .as_ref()
            .ok_or_else(|| super::error_codes::HOST_UNAVAILABLE.to_string())?;
        let preserved_marker = super::loading_marker::preserve();
        let recovery = forced_recovery.unwrap_or_else(|| match &preserved_marker.state {
            super::loading_marker::MarkerRead::Missing => {
                super::runtime_sync::RecoveryPreflight::Normal
            }
            super::loading_marker::MarkerRead::Valid(marker) => {
                super::runtime_sync::RecoveryPreflight::Interrupted(marker.extension_id.clone())
            }
            super::loading_marker::MarkerRead::Invalid => {
                super::runtime_sync::RecoveryPreflight::Invalid
            }
        });
        recovery.validate_retry_marker(&preserved_marker.state)?;
        let all_records = super::registry::list()?;
        let recovery = recovery.resolve_for(&all_records)?;
        let records = all_records
            .into_iter()
            .filter(|record| record.enabled && record.trusted)
            .collect();
        let build = super::runtime_sync::build_specs(records, &paths.directory, &recovery).await?;
        self.close_stale_channels(&build, deadline).await;
        let mut unavailable_ids = Vec::new();
        let mut responses =
            Vec::with_capacity(build.official_specs.len() + build.third_party_specs.len());
        if !build.official_specs.is_empty() {
            let api_level = super::runtime_host_load::official_api_level(&build.official_specs);
            if let Ok(process) = self
                .ensure_channel(HostIdentity::Official, api_level, deadline, start_reason)
                .await
            {
                let authorized = self.hosts.lock().await.authorize_loads(
                    &HostIdentity::Official,
                    &process,
                    &build.official_specs,
                );
                if authorized.is_err()
                    || super::runtime_host_load::load_specs(
                        &process,
                        &build.official_specs,
                        &mut responses,
                        &recovery,
                    )
                    .await
                    .is_err()
                {
                    let _ = self
                        .stop_host_if_current(
                            &HostIdentity::Official,
                            Some(&process),
                            deadline,
                            false,
                        )
                        .await;
                }
            } else {
                unavailable_ids.extend(
                    build
                        .official_specs
                        .iter()
                        .map(|specification| specification.id.clone()),
                );
            }
        }
        for (id, specification) in &build.third_party_specs {
            let identity = HostIdentity::ThirdParty(id.clone());
            let api_level = specification.manifest.api_level.clone();
            let Ok(process) = self
                .ensure_channel(identity, api_level, deadline, start_reason)
                .await
            else {
                unavailable_ids.push(id.clone());
                continue;
            };
            let identity = HostIdentity::ThirdParty(id.clone());
            let authorized = self.hosts.lock().await.authorize_loads(
                &identity,
                &process,
                std::slice::from_ref(specification),
            );
            if authorized.is_err()
                || super::runtime_host_load::load_specs(
                    &process,
                    std::slice::from_ref(specification),
                    &mut responses,
                    &recovery,
                )
                .await
                .is_err()
            {
                let identity = HostIdentity::ThirdParty(id.clone());
                let _ = self
                    .stop_host_if_current(&identity, Some(&process), deadline, false)
                    .await;
            }
        }
        let mut build = build;
        for id in unavailable_ids {
            build
                .failures
                .insert(id, super::error_codes::HOST_UNAVAILABLE.to_string());
        }
        let applied = super::runtime_sync::apply(responses, &build)?;
        super::loading_marker::complete(
            preserved_marker,
            &applied.completed_ids,
            recovery.retry_details(),
        )?;
        self.set_running(applied.active, applied.diagnostics);
        Ok(build.sensitive_access_reminder)
    }

    async fn close_stale_channels(
        &self,
        build: &super::runtime_sync::BuildSpecs,
        deadline: Instant,
    ) {
        let mut desired = build
            .third_party_specs
            .keys()
            .cloned()
            .map(HostIdentity::ThirdParty)
            .collect::<BTreeSet<_>>();
        if !build.official_specs.is_empty() {
            desired.insert(HostIdentity::Official);
        }
        let stale = self
            .hosts
            .lock()
            .await
            .bound_snapshots()
            .into_iter()
            .filter(|(identity, _, _)| !desired.contains(identity))
            .collect::<Vec<_>>();
        for (identity, _, process) in stale {
            let _ = self
                .stop_host_if_current(&identity, Some(&process), deadline, false)
                .await;
        }
    }
}

#[cfg(test)]
pub(super) use super::runtime_host_load::load_specs;

#[cfg(test)]
#[path = "runtime_channel_sync_tests.rs"]
mod tests;
