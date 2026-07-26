use crate::services::forecast::evaluation::{self, BacktestRequest};
use crate::services::forecast::sidecar::ChronosSidecar;
use crate::services::forecast::types::ForecastResult;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn run_forecast_backtest(
    app: AppHandle,
    request: BacktestRequest,
    chronos: State<'_, ChronosSidecar>,
) -> Result<ForecastResult, String> {
    let analysis = evaluation::run(request, chronos.inner()).await?;
    crate::services::forecast::events::emit_updated(&app, &analysis);
    Ok(analysis)
}

#[tauri::command]
pub async fn create_forecast_ensemble(
    app: AppHandle,
    analysis_id: String,
    model_ids: Vec<String>,
    chronos: State<'_, ChronosSidecar>,
) -> Result<ForecastResult, String> {
    let analysis = crate::services::forecast::advanced::ensemble::create(
        &analysis_id,
        &model_ids,
        Some(chronos.inner()),
    )
    .await?;
    crate::services::forecast::events::emit_updated(&app, &analysis);
    Ok(analysis)
}
