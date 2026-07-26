#[path = "tool_dispatcher_forecast_error.rs"]
mod forecast_error;

use self::forecast_error::model_error;
use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::types::ForecastRequest;
use crate::services::forecast::{data_profiles, registry, selected_model, selection_policy, validation};
use serde_json::Value;
use std::path::Path;
use std::time::Instant;
use tokio_util::sync::CancellationToken;

pub async fn handle(
    args: &Value,
    working_dir: &Path,
    session_id: &str,
    cancel: CancellationToken,
) -> ToolResult {
    if cancel.is_cancelled() {
        return ToolResult::err("Annulé.");
    }
    let started_at = Instant::now();
    let mut request: ForecastRequest = match serde_json::from_value(args.clone()) {
        Ok(request) => request,
        Err(_) => return ToolResult::err("Paramètres Forecast invalides"),
    };
    let requested_model = request.model.clone();
    crate::services::forecast::request_normalize::normalize_request(&mut request);
    let policy = match selection_policy::get() {
        Ok(policy) => policy,
        Err(error) => return model_error(None, "", requested_model.as_deref(), &error),
    };
    let selection_mode = policy.mode;
    let policy_model = policy.manual_model_id.clone().unwrap_or_default();
    let selected = match selected_model::apply_policy(&mut request, policy.clone()) {
        Ok(model) => model,
        Err(error) => {
            return model_error(
                Some(selection_mode),
                &policy_model,
                requested_model.as_deref(),
                &error,
            )
        }
    };
    if let Err(error) = data_profiles::hydrate_request(&mut request).await {
        return model_error(
            Some(selection_mode),
            &selected,
            requested_model.as_deref(),
            &error,
        );
    }
    if request.data.is_none() && request.file_path.is_none() {
        return model_error(
            Some(selection_mode),
            &selected,
            requested_model.as_deref(),
            "Il faut fournir data, file_path ou data_profile_id",
        );
    }
    if let Err(error) =
        crate::services::forecast::file_input::ensure_request_data(&mut request, Some(working_dir))
            .await
    {
        return model_error(
            Some(selection_mode),
            &selected,
            requested_model.as_deref(),
            &error,
        );
    }
    if let Err(error) = validation::validate_request(&request) {
        return model_error(
            Some(selection_mode),
            &selected,
            requested_model.as_deref(),
            &error,
        );
    }
    let data_profile =
        match crate::services::forecast::data_quality::validate_and_bind(&mut request) {
            Ok(profile) => profile,
            Err(error) => {
                return model_error(
                    Some(selection_mode),
                    &selected,
                    requested_model.as_deref(),
                    &error,
                )
            }
        };
    let selection_proof = match super::tool_dispatcher_forecast_selection::verify(
        &request,
        &data_profile,
        policy,
        session_id,
    )
    .await
    {
        Ok(proof) => proof,
        Err(error) => {
            return model_error(
                Some(selection_mode),
                &selected,
                requested_model.as_deref(),
                &error,
            )
        }
    };
    let (model_id, runtime) = match super::tool_dispatcher_forecast_runtime::resolve(
        &request,
        &data_profile,
    ) {
        Ok(runtime) => runtime,
        Err(error) => {
            return model_error(
                Some(selection_mode),
                &selected,
                requested_model.as_deref(),
                &error,
            )
        }
    };

    let result = if registry::is_cloud(runtime) {
        super::tool_dispatcher_forecast_execute::run_cloud(&request, session_id, cancel.clone())
            .await
    } else {
        super::tool_dispatcher_forecast_execute::run_local(
            &request,
            model_id,
            runtime,
            session_id,
            cancel.clone(),
        )
        .await
    };
    let mut forecast = match result {
        Ok(forecast) => forecast,
        Err(error) => {
            return model_error(
                Some(selection_mode),
                &selected,
                requested_model.as_deref(),
                &error,
            )
        }
    };
    let duration_ms = u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX);
    if let Err(error) = super::tool_dispatcher_forecast_selection::complete_provenance(
        &mut forecast,
        &request,
        &data_profile,
        selection_proof.as_ref(),
        duration_ms,
    ) {
        return model_error(
            Some(selection_mode),
            &selected,
            requested_model.as_deref(),
            &error,
        );
    }
    match super::tool_dispatcher_forecast_persist::save(
        &mut forecast,
        &request,
        &cancel,
    )
    .await
    {
        Ok(json) => ToolResult::ok(json),
        Err(error) => model_error(
            Some(selection_mode),
            &selected,
            requested_model.as_deref(),
            &error,
        ),
    }
}
