use super::host_identity::HostIdentity;
use super::host_process::HostProcess;
use super::protocol::{HelloResult, LoadResult};
use super::runtime::ExtensionRuntime;
use super::types::ExtensionApiLevel;
use serde_json::json;
use std::collections::BTreeSet;
use std::sync::Arc;

impl ExtensionRuntime {
    pub(super) async fn sync_hosts(&self) -> Result<(), String> {
        let _sync = self.sync.lock().await;
        let paths = self
            .paths
            .as_ref()
            .ok_or_else(|| super::error_codes::HOST_UNAVAILABLE.to_string())?;
        let build =
            super::runtime_sync::build_specs(super::registry::enabled_hosted()?, &paths.directory)?;
        self.close_stale_channels(&build).await;
        let mut responses =
            Vec::with_capacity(build.official_specs.len() + build.third_party_specs.len());
        if !build.official_specs.is_empty() {
            let api_level = official_api_level(&build.official_specs);
            if let Ok(process) = self.ensure_channel(HostIdentity::Official, api_level).await {
                if load_specs(&process, &build.official_specs, &mut responses)
                    .await
                    .is_err()
                {
                    let _ = self
                        .stop_channel(&HostIdentity::Official, Some(&process))
                        .await;
                }
            }
        }
        for (id, specification) in &build.third_party_specs {
            let identity = HostIdentity::ThirdParty(id.clone());
            let api_level = specification.manifest.api_level.clone();
            let Ok(process) = self.ensure_channel(identity, api_level).await else {
                continue;
            };
            if load_specs(
                &process,
                std::slice::from_ref(specification),
                &mut responses,
            )
            .await
            .is_err()
            {
                let identity = HostIdentity::ThirdParty(id.clone());
                let _ = self.stop_channel(&identity, Some(&process)).await;
            }
        }
        let applied = super::runtime_sync::apply(responses, &build)?;
        self.set_running(applied.active, applied.diagnostics);
        Ok(())
    }

    async fn ensure_channel(
        &self,
        identity: HostIdentity,
        api_level: ExtensionApiLevel,
    ) -> Result<Arc<HostProcess>, String> {
        let current = {
            let hosts = self.hosts.lock().await;
            hosts
                .snapshot(&identity)
                .map(|snapshot| (snapshot, hosts.usable_snapshot(&identity).is_some()))
        };
        if let Some(((current_level, _, process), usable)) = current {
            if usable && current_level == api_level && process.is_alive() {
                return Ok(process);
            }
            if !self.stop_channel(&identity, Some(&process)).await {
                return Err(super::error_codes::HOST_UNAVAILABLE.to_string());
            }
        }
        let reservation = self.hosts.lock().await.reserve(identity.clone())?;
        let paths = self
            .paths
            .as_ref()
            .ok_or_else(|| super::error_codes::HOST_UNAVAILABLE.to_string())?;
        let process = Arc::new(
            HostProcess::spawn_bound(
                paths,
                &self.work,
                identity,
                reservation.generation(),
                reservation.revoked(),
                reservation.temporary_directory(),
            )
            .await?,
        );
        let Ok(hello) = validate_hello(&process).await else {
            self.hosts.lock().await.revoke_reservation(&reservation);
            let _ = process.kill(super::host_process::stop_deadline()).await;
            return Err(super::error_codes::HOST_UNAVAILABLE.to_string());
        };
        self.set_host_version(&hello);
        self.hosts
            .lock()
            .await
            .bind(reservation, api_level, Arc::clone(&process))?;
        Ok(process)
    }

    async fn close_stale_channels(&self, build: &super::runtime_sync::BuildSpecs) {
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
            .snapshots()
            .into_iter()
            .filter(|(identity, _, _)| !desired.contains(identity))
            .collect::<Vec<_>>();
        for (identity, _, process) in stale {
            let _ = self.stop_channel(&identity, Some(&process)).await;
        }
    }
}

async fn load_specs(
    process: &HostProcess,
    specs: &[super::protocol::HostExtensionSpec],
    responses: &mut Vec<LoadResult>,
) -> Result<(), ()> {
    if process.request("host.reset", json!({})).await.is_err() {
        return Err(());
    }
    let mut loaded = Vec::with_capacity(specs.len());
    for specification in specs {
        match process
            .load(specification)
            .await
            .and_then(super::runtime::parse::<LoadResult>)
        {
            Ok(response) => loaded.push(response),
            Err(_) => return Err(()),
        }
    }
    responses.extend(loaded);
    Ok(())
}

async fn validate_hello(process: &HostProcess) -> Result<HelloResult, String> {
    let hello =
        super::runtime::parse::<HelloResult>(process.request("host.hello", json!({})).await?)?;
    if hello.api_version != super::types::BEAVER_API_VERSION {
        return Err(super::error_codes::HOST_INCOMPATIBLE.to_string());
    }
    super::runtime_version::validate_node(&hello.node_version)?;
    Ok(hello)
}

fn official_api_level(specs: &[super::protocol::HostExtensionSpec]) -> ExtensionApiLevel {
    if specs
        .iter()
        .any(|spec| spec.manifest.api_level == ExtensionApiLevel::Advanced)
    {
        ExtensionApiLevel::Advanced
    } else {
        ExtensionApiLevel::Stable
    }
}

#[cfg(test)]
#[path = "runtime_channel_sync_tests.rs"]
mod tests;
