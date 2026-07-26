use crate::services::forecast::types::{ForecastRequest, ForecastResult};
use crate::services::forecast::{
    client_chronos, client_nixtla, model_manager, registry, sidecar,
};
use tauri::Manager;
use tokio_util::sync::CancellationToken;

pub async fn run_cloud(
    request: &ForecastRequest,
    session_id: &str,
    cancel: CancellationToken,
) -> Result<ForecastResult, String> {
    if cancel.is_cancelled() {
        return Err("Annulé".to_string());
    }
    let key = crate::services::api_keys::get_key("nixtla")
        .map_err(|_| "Clé API Nixtla non configurée".to_string())?;
    tokio::select! {
        _ = cancel.cancelled() => Err("Annulé".to_string()),
        result = client_nixtla::predict(&key, request, Some(session_id)) => result,
    }
}

pub async fn run_local(
    request: &ForecastRequest,
    model_id: &str,
    runtime: &registry::ForecastRuntimeSpec,
    session_id: &str,
    cancel: CancellationToken,
) -> Result<ForecastResult, String> {
    if cancel.is_cancelled() {
        return Err("Annulé".to_string());
    }
    if !model_manager::is_installed(model_id) {
        return Err("Modèle non installé".into());
    }
    let spec = crate::services::forecast::catalog::find_model(model_id).ok_or("Modèle inconnu")?;
    crate::services::forecast::hardware_profile::validate_model_resources(spec)?;
    let app = super::app_handle_global::get().ok_or("Service de prédiction indisponible")?;
    let chronos = app.state::<sidecar::ChronosSidecar>();
    let _prediction_guard = tokio::select! {
        _ = cancel.cancelled() => return Err("Annulé".to_string()),
        guard = chronos.lock_prediction() => guard,
    };
    let endpoint = tokio::select! {
        _ = cancel.cancelled() => {
            sidecar::stop(chronos.inner()).await;
            return Err("Annulé".to_string());
        }
        result = sidecar::start(chronos.inner(), model_id, runtime.family_id) => {
            result.map_err(|_| "Impossible de démarrer le service de prédiction".to_string())?
        }
    };
    let result = tokio::select! {
        _ = cancel.cancelled() => {
            sidecar::stop(chronos.inner()).await;
            Err("Annulé".to_string())
        }
        result = client_chronos::predict(
            &endpoint.base_url,
            endpoint.auth_token.as_str(),
            request,
            Some(session_id),
        ) => result,
    };
    sidecar::schedule_idle_stop(chronos.inner());
    result
}
