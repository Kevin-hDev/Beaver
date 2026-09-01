use crate::services::forecast::evaluation::{self, BacktestRequest};
use crate::services::forecast::sidecar::ChronosSidecar;
use crate::services::forecast::types::ForecastResult;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn run_forecast_backtest(
    app: AppHandle,
    session_id: String,
    request: BacktestRequest,
    chronos: State<'_, ChronosSidecar>,
) -> Result<ForecastResult, String> {
    crate::services::forecast::storage::authorize_for_session(&session_id, &request.analysis_id)
        .await?;
    let sidecar = chronos.inner().clone();
    let operation_sidecar = sidecar.clone();
    let analysis = sidecar
        .run_cancellable(move || async move { evaluation::run(request, &operation_sidecar).await })
        .await?;
    crate::services::forecast::events::emit_updated(&app, &analysis);
    Ok(analysis)
}

#[tauri::command]
pub async fn create_forecast_ensemble(
    app: AppHandle,
    session_id: String,
    analysis_id: String,
    model_ids: Vec<String>,
    chronos: State<'_, ChronosSidecar>,
) -> Result<ForecastResult, String> {
    crate::services::forecast::storage::authorize_for_session(&session_id, &analysis_id).await?;
    let sidecar = chronos.inner().clone();
    let operation_sidecar = sidecar.clone();
    let analysis = sidecar
        .run_cancellable(move || async move {
            crate::services::forecast::advanced::ensemble::create(
                &analysis_id,
                &model_ids,
                Some(&operation_sidecar),
            )
            .await
        })
        .await?;
    crate::services::forecast::events::emit_updated(&app, &analysis);
    Ok(analysis)
}
