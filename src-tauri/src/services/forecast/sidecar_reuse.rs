use super::sidecar::{ChronosSidecar, SidecarEndpoint, SidecarHandle};
use super::sidecar_settings::LaunchSettings;
use std::time::Duration;
use zeroize::Zeroizing;

const HEALTH_PROBE_TIMEOUT: Duration = Duration::from_secs(3);

struct RunningIdentity {
    port: u16,
    pid: u32,
    model_id: String,
    family_id: String,
    launch: LaunchSettings,
    publication_generation: u64,
}

impl RunningIdentity {
    fn still_matches(&self, handle: &SidecarHandle) -> bool {
        handle.pid == self.pid
            && handle.model_id == self.model_id
            && handle.family_id == self.family_id
            && handle.launch == self.launch
            && handle.publication_generation == self.publication_generation
    }
}

pub(super) async fn reuse_running(
    sidecar: &ChronosSidecar,
    model_name: &str,
    family_id: &str,
    launch: &LaunchSettings,
) -> Option<SidecarEndpoint> {
    reuse_running_with_probe(sidecar, model_name, family_id, launch, |port, token| {
        super::sidecar_http::health_info(port, &token)
    })
    .await
}

async fn reuse_running_with_probe<Probe>(
    sidecar: &ChronosSidecar,
    model_name: &str,
    family_id: &str,
    launch: &LaunchSettings,
    probe: Probe,
) -> Option<SidecarEndpoint>
where
    Probe: FnOnce(u16, Zeroizing<String>) -> Option<(u16, String, String)> + Send + 'static,
{
    let (identity, probe_token) = {
        let guard = sidecar.process.lock().await;
        let handle = guard.as_ref()?;
        if handle.model_id != model_name
            || handle.family_id != family_id
            || &handle.launch != launch
        {
            return None;
        }
        (
            RunningIdentity {
                port: super::sidecar_http::get_port(),
                pid: handle.pid,
                model_id: handle.model_id.clone(),
                family_id: handle.family_id.clone(),
                launch: handle.launch.clone(),
                publication_generation: handle.publication_generation,
            },
            handle.auth_token.clone(),
        )
    };
    let probe_port = identity.port;
    let health = tokio::time::timeout(
        HEALTH_PROBE_TIMEOUT,
        tokio::task::spawn_blocking(move || probe(probe_port, probe_token)),
    )
    .await
    .ok()?
    .ok()??;
    let (ready_port, ready_model, ready_family) = health;
    if ready_port != identity.port || ready_model != model_name || ready_family != family_id {
        return None;
    }

    let mut guard = sidecar.process.lock().await;
    let handle = guard.as_mut()?;
    if !identity.still_matches(handle) {
        return None;
    }
    handle.generation = handle.generation.saturating_add(1);
    Some(SidecarEndpoint {
        base_url: format!("http://127.0.0.1:{ready_port}"),
        auth_token: handle.auth_token.clone(),
        pid: handle.pid,
    })
}

#[cfg(test)]
impl ChronosSidecar {
    pub(crate) async fn probe_running_for_test<Probe>(&self, probe: Probe) -> bool
    where
        Probe: FnOnce(u16, Zeroizing<String>) -> Option<(u16, String, String)> + Send + 'static,
    {
        reuse_running_with_probe(
            self,
            "fixture",
            "fixture",
            &super::sidecar_settings::current(),
            probe,
        )
        .await
        .is_some()
    }
}
