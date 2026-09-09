use serde_json::Value;

pub(super) fn supported_parameters(model: &Value) -> Option<Option<Vec<String>>> {
    let Some(value) = model.get("supported_parameters") else {
        return Some(None);
    };
    if value.is_null() {
        return Some(None);
    }
    let values = value.as_array()?;
    if values.len() > 64 {
        return None;
    }
    values
        .iter()
        .map(|value| safe_text(value, 64))
        .collect::<Option<Vec<_>>>()
        .map(Some)
}

pub(super) fn supports_text_output(model: &Value) -> bool {
    let value = &model["architecture"]["output_modalities"];
    if value.is_null() {
        return true;
    }
    let Some(values) = value.as_array().filter(|values| values.len() <= 8) else {
        return false;
    };
    values
        .iter()
        .map(|value| safe_text(value, 32))
        .collect::<Option<Vec<_>>>()
        .is_some_and(|modalities| modalities.iter().any(|modality| modality == "text"))
}

pub(super) fn safe_owner(value: &Value) -> Option<String> {
    safe_text(value, 96)
}

fn safe_text(value: &Value, max_bytes: usize) -> Option<String> {
    value
        .as_str()
        .filter(|text| {
            !text.is_empty() && text.len() <= max_bytes && !text.chars().any(char::is_control)
        })
        .map(str::to_string)
}
