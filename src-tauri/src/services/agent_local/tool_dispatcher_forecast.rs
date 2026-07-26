use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::storage;
use serde_json::Value;
use std::path::Path;
use tokio_util::sync::CancellationToken;

pub async fn dispatch_forecast(
    tool_name: &str,
    args: &Value,
    working_dir: &Path,
    session_id: &str,
    cancel: CancellationToken,
) -> Option<ToolResult> {
    match tool_name {
        "forecast" => Some(
            super::tool_dispatcher_forecast_run::handle(args, working_dir, session_id, cancel)
                .await,
        ),
        "forecast_models" => Some(
            super::tool_dispatcher_forecast_models::handle(args, session_id).await,
        ),
        "forecast_analyze" => Some(super::tool_dispatcher_forecast_analyze::handle(args).await),
        "forecast_data_audit" => Some(
            super::tool_dispatcher_forecast_data_audit::handle(args, working_dir).await,
        ),
        "forecast_read" => Some(handle_read(args).await),
        "forecast_backtest" => {
            Some(super::tool_dispatcher_forecast_evaluation::backtest(args).await)
        }
        "forecast_compare_models" => {
            Some(super::tool_dispatcher_forecast_evaluation::compare(args).await)
        }
        _ => None,
    }
}

async fn handle_read(args: &Value) -> ToolResult {
    match args["analysis_id"].as_str() {
        Some(id) if !id.trim().is_empty() => match storage::load(id.trim()).await {
            Ok(analysis) => {
                let offset = args["offset"].as_u64().unwrap_or(0) as usize;
                let limit = args["limit"].as_u64().unwrap_or(100) as usize;
                match super::tool_dispatcher_forecast_output::analysis_payload(
                    &analysis, offset, limit,
                ) {
                    Ok(json) => ToolResult::ok(json),
                    Err(error) => ToolResult::err(error),
                }
            }
            Err(error) => ToolResult::err(error),
        },
        _ => match storage::list().await {
            Ok(list) => match super::tool_dispatcher_forecast_output::list_payload(&list) {
                Ok(json) => ToolResult::ok(json),
                Err(error) => ToolResult::err(error),
            },
            Err(error) => ToolResult::err(error),
        },
    }
}
