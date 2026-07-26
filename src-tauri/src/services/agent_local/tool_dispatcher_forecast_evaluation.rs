use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::evaluation::BacktestRequest;
use crate::services::forecast::sidecar::ChronosSidecar;
use crate::services::forecast::storage;
use serde_json::Value;
use tauri::Manager;

pub async fn backtest(args: &Value) -> ToolResult {
    let request: BacktestRequest = match serde_json::from_value(args.clone()) {
        Ok(request) => request,
        Err(_) => return ToolResult::err("Paramètres de backtest invalides"),
    };
    let Some(app) = super::app_handle_global::get() else {
        return ToolResult::err("Service de backtest indisponible");
    };
    let chronos = app.state::<ChronosSidecar>();
    match crate::services::forecast::evaluation::run(request, chronos.inner()).await {
        Ok(analysis) => {
            crate::services::forecast::events::emit_updated(app, &analysis);
            comparison_payload(&analysis)
        }
        Err(error) => ToolResult::err(error),
    }
}

pub async fn compare(args: &Value) -> ToolResult {
    let Some(id) = args["analysis_id"].as_str().filter(|id| !id.trim().is_empty()) else {
        return ToolResult::err("Identifiant d'analyse requis");
    };
    match storage::load(id.trim()).await {
        Ok(analysis) => comparison_payload(&analysis),
        Err(error) => ToolResult::err(error),
    }
}

fn comparison_payload(analysis: &crate::services::forecast::types::ForecastResult) -> ToolResult {
    let Some(evaluation) = &analysis.evaluation else {
        return ToolResult::err("Aucun backtest comparable pour cette analyse");
    };
    let mut results: Vec<_> = evaluation.results.iter().collect();
    results.sort_by_key(|result| result.rank.unwrap_or(usize::MAX));
    let model_failures: Vec<_> = results
        .iter()
        .filter(|result| {
            result.kind == crate::services::forecast::evaluation::types::BacktestKind::Model
        })
        .filter_map(|result| {
            result.failure.as_ref().map(|failure| {
                serde_json::json!({
                    "model_id": result.model_id,
                    "failure": failure,
                })
            })
        })
        .collect();
    let baseline_failures: Vec<_> = results
        .iter()
        .filter(|result| {
            result.kind == crate::services::forecast::evaluation::types::BacktestKind::Baseline
        })
        .filter_map(|result| {
            result.failure.as_ref().map(|failure| {
                serde_json::json!({
                    "model_id": result.model_id,
                    "failure": failure,
                })
            })
        })
        .collect();
    let has_failures = evaluation
        .results
        .iter()
        .any(|result| result.failure.is_some());
    let payload = serde_json::json!({
        "analysis_id": analysis.id,
        "status": if has_failures { "partial" } else { "complete" },
        "ranking_method": "rolling_backtest",
        "horizon": evaluation.horizon,
        "windows": evaluation.windows,
        "warning": evaluation.warning,
        "results": results.into_iter().map(|result| serde_json::json!({
            "model_id": result.model_id,
            "kind": result.kind,
            "rank": result.rank,
            "metrics": result.metrics,
            "calibration": result.calibration,
            "beats_best_baseline": result.beats_best_baseline,
            "duration_ms": result.duration_ms,
            "max_memory_mb": result.max_memory_mb,
            "failure": result.failure,
        })).collect::<Vec<_>>(),
        "model_failures": model_failures,
        "baseline_failures": baseline_failures,
        "usage": if has_failures {
            "This backtest is partial. Report failed entries and their structured failure codes; compare only successful results and do not claim that every requested model or baseline was validated."
        } else {
            "Follow rank among successful results. Ranking prioritizes MASE, then quantile loss, interval coverage, other accuracy metrics, stability, observed memory and duration. Require an advanced model to beat the best baseline."
        }
    });
    match serde_json::to_string(&payload) {
        Ok(json) => ToolResult::ok(json),
        Err(_) => ToolResult::err("Résultat de comparaison indisponible"),
    }
}

#[cfg(test)]
#[path = "tool_dispatcher_forecast_evaluation_tests.rs"]
mod tests;
