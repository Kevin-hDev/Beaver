use serde_json::Value;

const MAX_RUNNING_MODELS: usize = 32;
const MAX_CONTEXT_LENGTH: u64 = 16 * 1024 * 1024;

pub fn loaded_context_length(body: &[u8], selected_model: &str) -> Result<Option<u64>, String> {
    let json: Value = serde_json::from_slice(body).map_err(|_| "ollama-invalid-response")?;
    let models = json["models"]
        .as_array()
        .ok_or_else(|| "ollama-invalid-response".to_string())?;

    Ok(models
        .iter()
        .take(MAX_RUNNING_MODELS)
        .find(|model| matches_model(model, selected_model))
        .and_then(|model| model["context_length"].as_u64())
        .filter(|length| *length > 0 && *length <= MAX_CONTEXT_LENGTH))
}

fn matches_model(model: &Value, selected: &str) -> bool {
    [model["name"].as_str(), model["model"].as_str()]
        .into_iter()
        .flatten()
        .any(|candidate| same_model_name(candidate, selected))
}

fn same_model_name(left: &str, right: &str) -> bool {
    left == right
        || left.strip_suffix(":latest") == Some(right)
        || right.strip_suffix(":latest") == Some(left)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_loaded_context_for_selected_model() {
        let body = br#"{"models":[
            {"name":"other:latest","context_length":4096},
            {"name":"gemma4:e2b","context_length":65536}
        ]}"#;
        assert_eq!(
            loaded_context_length(body, "gemma4:e2b").unwrap(),
            Some(65_536)
        );
    }

    #[test]
    fn matches_implicit_latest_tag() {
        let body = br#"{"models":[{"model":"llama3:latest","context_length":8192}]}"#;
        assert_eq!(loaded_context_length(body, "llama3").unwrap(), Some(8192));
    }

    #[test]
    fn rejects_unreasonable_context_and_bounds_models() {
        let mut models = vec![serde_json::json!({
            "name": "too-large",
            "context_length": MAX_CONTEXT_LENGTH + 1
        })];
        models.extend((0..MAX_RUNNING_MODELS).map(|index| {
            serde_json::json!({"name": format!("model-{index}"), "context_length": 4096})
        }));
        models.push(serde_json::json!({
            "name": "outside-limit",
            "context_length": 8192
        }));
        let body = serde_json::to_vec(&serde_json::json!({"models": models})).unwrap();

        assert_eq!(loaded_context_length(&body, "too-large").unwrap(), None);
        assert_eq!(loaded_context_length(&body, "outside-limit").unwrap(), None);
    }
}
