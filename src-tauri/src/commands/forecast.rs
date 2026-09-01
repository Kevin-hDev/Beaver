use crate::services::forecast::types::{ForecastAnalysisMeta, ForecastRequest, ForecastResult};
use crate::services::forecast::{
    catalog, client_chronos, client_nixtla, data_profiles, export, model_manager, notes_cleanup,
    registry, selected_model, sidecar, storage, validation,
};
use std::time::Instant;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn run_forecast(
    session_id: String,
    request: ForecastRequest,
    chronos: State<'_, sidecar::ChronosSidecar>,
) -> Result<ForecastResult, String> {
    let sidecar = chronos.inner().clone();
    let operation_sidecar = sidecar.clone();
    sidecar
        .run_cancellable(move || run_forecast_inner(session_id, request, operation_sidecar))
        .await
}

async fn run_forecast_inner(
    session_id: String,
    mut request: ForecastRequest,
    chronos: sidecar::ChronosSidecar,
) -> Result<ForecastResult, String> {
    let started_at = Instant::now();
    crate::services::forecast::request_normalize::normalize_request(&mut request);
    let policy = crate::services::forecast::selection_policy::get()?;
    let selection_mode = policy.mode;
    selected_model::apply_frontend_policy(&mut request, policy.clone())?;
    data_profiles::hydrate_request(&session_id, &mut request).await?;
    crate::services::forecast::file_input::ensure_request_data(&mut request, None)
        .await
        .map_err(|_| "Impossible de lire les données source".to_string())?;
    validation::validate_request(&request)?;
    let data_profile = crate::services::forecast::data_quality::validate_and_bind(&mut request)?;
    let model_id = validation::model_id(&request)?.to_string();
    let selection_proof = if selection_mode
        == crate::services::forecast::selection_policy::ForecastSelectionMode::Auto
    {
        request.selection_id = None;
        request.selection_source = Some(
            crate::services::forecast::provenance_types::ForecastSelectionSource::ExplicitUserOverride,
        );
        request.selection_reason_codes = vec!["user_requested".into()];
        Some(
            crate::services::forecast::auto_selection_ui::verify_choice(
                &session_id,
                &data_profile,
                &policy,
                &model_id,
            )
            .await?,
        )
    } else {
        None
    };
    let spec = catalog::find_model(&model_id).ok_or("Modèle inconnu")?;
    let runtime = registry::find_runtime(&model_id).ok_or("Moteur indisponible")?;
    validate_future_context(&request, &data_profile, runtime)?;
    if !registry::has_predict_adapter(runtime) {
        return Err("Moteur indisponible".into());
    }

    let mut result = if registry::is_cloud(runtime) {
        let key = crate::services::api_keys::get_key("nixtla")
            .map_err(|_| "Clé API Nixtla non configurée".to_string())?;
        client_nixtla::predict(&key, &request, Some(&session_id))
            .await
            .map_err(|_| "Erreur du service de prédiction".to_string())?
    } else {
        if !model_manager::is_ready(&model_id) {
            return Err("Modèle non installé".into());
        }
        crate::services::forecast::hardware_profile::validate_model_resources(spec)?;
        let _prediction_guard = chronos.lock_prediction().await;
        let endpoint = sidecar::start(&chronos, &model_id, runtime.family_id)
            .await
            .map_err(|_| "Impossible de démarrer le service de prédiction".to_string())?;
        let prediction = client_chronos::predict(
            &endpoint.base_url,
            endpoint.auth_token.as_str(),
            &request,
            Some(&session_id),
        )
        .await;
        sidecar::schedule_idle_stop(&chronos).await;
        prediction.map_err(|_| "Erreur du service de prédiction".to_string())?
    };

    crate::services::forecast::provenance::complete(
        &mut result,
        &request,
        &data_profile,
        request.selection_source.unwrap_or(
            crate::services::forecast::provenance_types::ForecastSelectionSource::Manual,
        ),
        selection_proof.as_ref(),
        u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX),
    )?;

    if let Some(profile) = &result.data_profile {
        data_profiles::save(&session_id, profile, &request).await?;
    }
    storage::save_for_session(&session_id, &mut result).await?;
    Ok(result)
}

fn validate_future_context(
    request: &ForecastRequest,
    profile: &crate::services::forecast::data_quality::DataProfile,
    runtime: &registry::ForecastRuntimeSpec,
) -> Result<(), String> {
    if profile.future_rows > 0
        && !request.covariate_columns.is_empty()
        && !runtime.capabilities.future_covariates
    {
        return Err("Variables futures non supportées par ce moteur".into());
    }
    Ok(())
}

#[tauri::command]
pub async fn list_forecast_analyses(
    session_id: String,
) -> Result<Vec<ForecastAnalysisMeta>, String> {
    storage::list_for_session(&session_id).await
}

#[tauri::command]
pub async fn list_unassigned_forecast_analyses(
    session_id: String,
) -> Result<Vec<ForecastAnalysisMeta>, String> {
    storage::list_unassigned_for_session(&session_id).await
}

#[tauri::command]
pub async fn claim_legacy_forecast_analysis(
    app: AppHandle,
    session_id: String,
    id: String,
) -> Result<ForecastResult, String> {
    let analysis = storage::claim_legacy_for_session(&session_id, &id).await?;
    crate::services::forecast::events::emit_created(&app, &analysis);
    Ok(analysis)
}

#[tauri::command]
pub async fn get_forecast_analysis(
    session_id: String,
    id: String,
) -> Result<ForecastResult, String> {
    storage::load_for_session(&session_id, &id).await
}

#[tauri::command]
pub async fn export_forecast_analysis(
    session_id: String,
    analysis_id: String,
    format: String,
) -> Result<export::ForecastExportResult, String> {
    storage::authorize_for_session(&session_id, &analysis_id).await?;
    export::export_analysis(&analysis_id, &format).await
}

#[tauri::command]
pub async fn delete_forecast_analysis(
    app: AppHandle,
    session_id: String,
    id: String,
) -> Result<(), String> {
    storage::authorize_for_session(&session_id, &id).await?;
    let workspace = crate::services::workspace_scope::resolve(&session_id).await?;
    notes_cleanup::delete_analysis(&id).await?;
    crate::services::forecast::events::emit_deleted(&app, &id, &workspace);
    Ok(())
}

#[tauri::command]
pub async fn rename_forecast_analysis(
    app: AppHandle,
    session_id: String,
    id: String,
    name: String,
) -> Result<ForecastAnalysisMeta, String> {
    let renamed = storage::rename_for_session(&session_id, &id, &name).await?;
    crate::services::forecast::events::emit_updated(&app, &renamed);
    Ok(renamed.to_meta())
}
