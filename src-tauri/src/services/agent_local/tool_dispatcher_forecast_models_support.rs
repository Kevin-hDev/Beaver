use serde_json::Value;

pub(super) fn requested_model_id(args: &Value) -> Result<Option<&str>, String> {
    match args.get("requested_model_id") {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(id)) => {
            let id = id.trim();
            if id.is_empty() {
                return Ok(None);
            }
            crate::services::forecast::validation::validate_model_id_format(id)?;
            Ok(Some(id))
        }
        Some(_) => Err("Modèle demandé invalide".to_string()),
    }
}

pub(super) fn compact_model(model: &Value, forced_model: Option<&str>) -> Option<Value> {
    let id = model["id"].as_str()?;
    Some(serde_json::json!({
        "id": id,
        "selected": forced_model == Some(id),
        "name": model["display_name"].as_str().unwrap_or(""),
        "provider": model["provider_id"].as_str().unwrap_or(""),
        "family": model["family_id"].as_str().unwrap_or(""),
        "installed": model["installed"].as_bool().unwrap_or(false),
        "runnable": model["runnable"].as_bool().unwrap_or(false),
        "runtime_ready": model["runtime_ready"].as_bool().unwrap_or(false),
        "provider_configured": model["provider_configured"].as_bool().unwrap_or(false),
        "is_cloud": model["is_cloud"].as_bool().unwrap_or(false),
        "interval_support": crate::services::forecast::validation::interval_support(id),
        "interval_capability": crate::services::forecast::interval_capability::for_model(id),
        "capabilities": model["capabilities"].clone()
    }))
}

pub(super) fn model_sort_key(model: &Value) -> (bool, bool, bool, String) {
    (
        !model["selected"].as_bool().unwrap_or(false),
        !model["runnable"].as_bool().unwrap_or(false),
        !model["installed"].as_bool().unwrap_or(false),
        model["id"].as_str().unwrap_or_default().to_string(),
    )
}
