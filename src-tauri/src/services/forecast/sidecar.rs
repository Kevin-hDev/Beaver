use crate::services::forecast::{
    sidecar_auth, sidecar_http, sidecar_process,
    sidecar_settings::{self, LaunchSettings, UnloadPolicy},
    sidecar_spawn,
};
use crate::services::paths::data_dir;
use std::process::Child;
use std::sync::Arc;
use tokio::sync::Mutex;
use zeroize::Zeroizing;

struct SidecarHandle {
    child: Child,
    model_id: String,
    family_id: String,
    auth_token: Zeroizing<String>,
    launch: LaunchSettings,
    generation: u64,
}

pub struct ChronosSidecar {
    process: Arc<Mutex<Option<SidecarHandle>>>,
    prediction: Mutex<()>,
}

pub struct SidecarEndpoint {
    pub base_url: String,
    pub auth_token: Zeroizing<String>,
    pub pid: u32,
}

impl ChronosSidecar {
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            prediction: Mutex::new(()),
        }
    }

    pub async fn lock_prediction(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.prediction.lock().await
    }
}

pub fn get_port() -> u16 {
    sidecar_http::get_port()
}

pub fn base_url() -> String {
    sidecar_http::base_url()
}

pub async fn start(
    sidecar: &ChronosSidecar,
    model_name: &str,
    family_id: &str,
) -> Result<SidecarEndpoint, String> {
    let launch = sidecar_settings::current();
    if let Some(endpoint) = reuse_running(sidecar, model_name, family_id, &launch).await {
        return Ok(endpoint);
    }

    stop(sidecar).await;
    sidecar_process::kill_orphan_sidecar();
    let port = sidecar_http::find_free_port();
    let script = sidecar_spawn::sidecar_dir().join("server.py");
    if !script.exists() {
        return Err("Sidecar Python non installé".into());
    }

    let runtime_python = sidecar_spawn::ready_runtime(family_id)?;
    let models_dir = data_dir().join("forecast-models");
    let auth_token = sidecar_auth::generate_auth_token();
    let child = sidecar_spawn::spawn_process(
        runtime_python,
        &script,
        port,
        model_name,
        family_id,
        &models_dir,
        &auth_token,
        &launch,
    )?;

    let pid = child.id();
    sidecar_process::save_pid(pid);
    sidecar_http::set_port(port);
    *sidecar.process.lock().await = Some(SidecarHandle {
        child,
        model_id: model_name.to_string(),
        family_id: family_id.to_string(),
        auth_token: auth_token.clone(),
        launch,
        generation: 1,
    });

    match sidecar_spawn::wait_until_ready(port, model_name, family_id, pid, auth_token).await {
        Ok(endpoint) => Ok(endpoint),
        Err(err) => {
            stop(sidecar).await;
            Err(err)
        }
    }
}

async fn reuse_running(
    sidecar: &ChronosSidecar,
    model_name: &str,
    family_id: &str,
    launch: &LaunchSettings,
) -> Option<SidecarEndpoint> {
    let mut guard = sidecar.process.lock().await;
    let handle = guard.as_mut()?;
    if handle.model_id != model_name || handle.family_id != family_id || &handle.launch != launch {
        return None;
    }
    let (_port, model, family) = sidecar_http::health_info(get_port(), handle.auth_token.as_str())?;
    if model != model_name || family != family_id {
        return None;
    }
    handle.generation = handle.generation.saturating_add(1);
    Some(SidecarEndpoint {
        base_url: base_url(),
        auth_token: handle.auth_token.clone(),
        pid: handle.child.id(),
    })
}

pub fn schedule_idle_stop(sidecar: &ChronosSidecar) {
    let state = sidecar.process.clone();
    tokio::spawn(async move {
        let (generation, policy) = match touch_state(&state).await {
            Some(item) => item,
            None => return,
        };
        let UnloadPolicy::After(delay) = policy else {
            return;
        };
        tokio::time::sleep(delay).await;
        stop_if_generation(&state, generation).await;
    });
}

async fn touch_state(state: &Arc<Mutex<Option<SidecarHandle>>>) -> Option<(u64, UnloadPolicy)> {
    let mut guard = state.lock().await;
    let handle = guard.as_mut()?;
    handle.generation = handle.generation.saturating_add(1);
    Some((handle.generation, handle.launch.unload_policy.clone()))
}

async fn stop_if_generation(state: &Arc<Mutex<Option<SidecarHandle>>>, generation: u64) {
    let should_stop = state
        .lock()
        .await
        .as_ref()
        .is_some_and(|handle| handle.generation == generation);
    if should_stop {
        stop_state(state).await;
    }
}

pub async fn stop(sidecar: &ChronosSidecar) {
    stop_state(&sidecar.process).await;
}

pub async fn stop_model(sidecar: &ChronosSidecar, model_id: &str) {
    let handle = {
        let mut state = sidecar.process.lock().await;
        if state
            .as_ref()
            .is_some_and(|handle| handle.model_id == model_id)
        {
            state.take()
        } else {
            None
        }
    };
    if let Some(handle) = handle {
        stop_handle(handle).await;
        sidecar_process::clear_pid_file();
        sidecar_http::clear_port();
    }
}

async fn stop_state(state: &Arc<Mutex<Option<SidecarHandle>>>) {
    if let Some(handle) = state.lock().await.take() {
        stop_handle(handle).await;
    }
    sidecar_process::clear_pid_file();
    sidecar_http::clear_port();
}

async fn stop_handle(handle: SidecarHandle) {
    let _ = tokio::task::spawn_blocking(move || {
        sidecar_process::kill_child_process(handle.child);
    })
    .await;
}
