use crate::services::forecast::{
    sidecar_http, sidecar_process_env, sidecar_runtime, sidecar_settings::LaunchSettings,
};
use crate::services::paths::data_dir;
use std::path::{Path, PathBuf};
use std::process::Child;
use zeroize::Zeroizing;

use super::sidecar::SidecarEndpoint;

pub fn sidecar_dir() -> PathBuf {
    data_dir().join("forecast-sidecar")
}

pub fn ready_runtime(family_id: &str) -> Result<PathBuf, String> {
    sidecar_runtime::ensure_runtime(&sidecar_dir(), family_id)
        .map_err(|_| "Moteur Forecast non préparé".to_string())
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_process(
    runtime_python: PathBuf,
    script: &Path,
    port: u16,
    model_name: &str,
    family_id: &str,
    models_dir: &Path,
    auth_token: &str,
    launch: &LaunchSettings,
) -> Result<Child, String> {
    let mut cmd = std::process::Command::new(runtime_python);
    sidecar_process_env::configure(&mut cmd, &sidecar_dir())?;
    cmd.args([
        script.to_str().unwrap_or("server.py"),
        "--port",
        &port.to_string(),
        "--model",
        model_name,
        "--family",
        family_id,
        "--models-dir",
        models_dir.to_str().unwrap_or(""),
    ])
    .env("CLGO_FORECAST_TOKEN", auth_token)
    .env("TABPFN_DISABLE_TELEMETRY", "1")
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null());
    for (key, value) in launch.env_vars() {
        cmd.env(key, value);
    }
    cmd.spawn()
        .map_err(|_| "Impossible de lancer le sidecar Forecast".to_string())
}

pub async fn wait_until_ready(
    port: u16,
    model_name: &str,
    family_id: &str,
    pid: u32,
    auth_token: Zeroizing<String>,
) -> Result<SidecarEndpoint, String> {
    for _ in 0..60 {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if let Some((ready_port, ready_model, ready_family)) =
            sidecar_http::health_info(port, auth_token.as_str())
        {
            if ready_model == model_name && ready_family == family_id {
                return Ok(SidecarEndpoint {
                    base_url: format!("http://127.0.0.1:{ready_port}"),
                    auth_token,
                    pid,
                });
            }
        }
    }
    Err("Sidecar Forecast: timeout au démarrage".into())
}
