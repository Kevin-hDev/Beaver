use serde_json::Value;

const MAX_REASONING_EFFORTS: usize = 8;
/// One authority for the gateway catalog's priority over upstream defaults.
pub(super) fn owns_catalog_metadata(provider_id: &str) -> bool {
    provider_id == "openrouter"
}

pub(super) fn reasoning(value: &Value) -> Option<(Vec<String>, Option<String>)> {
    let object = value.as_object()?;
    let mandatory = object
        .get("mandatory")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let efforts = object.get("supported_efforts")?.as_array()?;
    let mut modes = Vec::with_capacity(MAX_REASONING_EFFORTS);
    for effort in efforts.iter().take(MAX_REASONING_EFFORTS) {
        let Some(effort) = ui_effort(effort) else {
            continue;
        };
        // Only expose modes the existing OpenRouter transport can encode.
        if (effort != "auto"
            && crate::services::reasoning::openrouter_effort(Some(effort)).is_none())
            || (mandatory && effort == "off")
            || modes.iter().any(|mode| mode == effort)
        {
            continue;
        }
        modes.push(effort.to_string());
    }
    let default_mode = object
        .get("default_effort")
        .and_then(ui_effort)
        .filter(|effort| modes.iter().any(|mode| mode == effort))
        .map(str::to_string);
    Some((modes, default_mode))
}

fn ui_effort(value: &Value) -> Option<&str> {
    // OpenRouter publishes `none`; Beaver's existing wire adapter calls it `off`.
    value
        .as_str()
        .map(|effort| if effort == "none" { "off" } else { effort })
}
