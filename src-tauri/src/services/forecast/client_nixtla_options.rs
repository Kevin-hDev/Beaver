use serde_json::{Map, Value};

pub fn effective_config(model: &str) -> Map<String, Value> {
    crate::services::forecast::model_config::effective_values(model).unwrap_or_default()
}

pub fn effective_level(confidence: f64) -> u64 {
    (confidence * 100.0).round() as u64
}

pub fn interval_array(body: &Value, key: &str) -> Result<Vec<f64>, String> {
    let values = body["intervals"][key]
        .as_array()
        .or_else(|| body[key].as_array())
        .ok_or("Intervalles Nixtla manquants")?;
    values
        .iter()
        .map(|value| {
            value
                .as_f64()
                .filter(|number| number.is_finite())
                .ok_or("Intervalle Nixtla invalide".to_string())
        })
        .collect()
}

pub fn apply(payload: &mut Value, config: &Map<String, Value>) {
    let Some(object) = payload.as_object_mut() else {
        return;
    };
    for key in [
        "clean_ex_first",
        "finetune_steps",
        "finetune_loss",
        "finetune_depth",
        "feature_contributions",
    ] {
        if let Some(value) = config.get(key) {
            object.insert(key.to_string(), value.clone());
        }
    }
}

#[cfg(test)]
#[path = "client_nixtla_options_tests.rs"]
mod tests;
