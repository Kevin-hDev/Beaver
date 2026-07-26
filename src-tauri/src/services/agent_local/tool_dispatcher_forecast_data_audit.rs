use crate::services::agent_local::types_tools::ToolResult;
use crate::services::forecast::types::ForecastRequest;
use crate::services::forecast::{data_profiles, data_quality, file_input, validation};
use serde_json::Value;
use std::path::Path;

pub async fn handle(args: &Value, working_dir: &Path) -> ToolResult {
    let mut request: ForecastRequest = match serde_json::from_value(args.clone()) {
        Ok(request) => request,
        Err(_) => return ToolResult::err("Paramètres d'audit invalides"),
    };
    crate::services::forecast::request_normalize::normalize_request(&mut request);
    if request.data_profile_id.is_some() {
        return ToolResult::err("Fournir des données à auditer, pas un profil existant");
    }
    if let Err(error) = file_input::ensure_request_data(&mut request, Some(working_dir)).await {
        return ToolResult::err(error);
    }
    if let Err(error) = validation::validate_data_request(&request) {
        return ToolResult::err(error);
    }
    let (_, profile) = match data_quality::audit_request_data(&request) {
        Ok(result) => result,
        Err(error) => return ToolResult::err(error),
    };
    if profile.valid {
        if let Err(error) = data_profiles::save(&profile, &request).await {
            return ToolResult::err(error);
        }
    }
    let payload = serde_json::json!({
        "status": if profile.valid { "valid" } else { "invalid" },
        "data_profile_id": profile.valid.then_some(profile.id.as_str()),
        "profile": profile,
        "next_step": if profile.valid {
            "Call forecast_models next. In Auto, choose only a returned confidence-compatible candidate. In Manual, verify forced_model_state.interval_capability. Then use the same data_profile_id and exact confidence_level in forecast; never round it."
        } else {
            "Correct every error issue, then run forecast_data_audit again."
        }
    });
    serde_json::to_string_pretty(&payload)
        .map_or_else(|_| ToolResult::err("Résultat d'audit invalide"), ToolResult::ok)
}
