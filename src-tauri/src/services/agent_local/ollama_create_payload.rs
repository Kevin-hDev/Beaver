use serde_json::Value;

pub fn system_prompt(
    model_name: &str,
    current_modelfile: &str,
    prompt: &str,
) -> Result<Value, String> {
    super::model_customizations::validate_model_name(model_name)?;
    let normalized = super::ollama_modelfile_system::normalize_prompt(prompt)?.unwrap_or_default();
    let mut parsed = super::modelfile_parser::parse_modelfile(current_modelfile);
    parsed.from = Some(model_name.to_string());
    parsed.system = Some(normalized);
    parsed.license = None;
    Ok(parsed.to_api_payload(model_name))
}

pub fn non_streaming(payload: &Value) -> Result<Value, String> {
    let mut enriched = payload.clone();
    let object = enriched
        .as_object_mut()
        .ok_or_else(|| "ollama-create-error".to_string())?;
    object.insert("stream".into(), serde_json::json!(false));
    Ok(enriched)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn system_prompt_inherits_model_and_keeps_supported_settings() {
        let current = r#"FROM /models/blobs/sha256-old
SYSTEM "old"
TEMPLATE "template"
LICENSE "license"
PARAMETER temperature 0.7"#;

        let payload = system_prompt("gemma4:e2b", current, "  New behavior  ").unwrap();

        assert_eq!(payload["model"], json!("gemma4:e2b"));
        assert_eq!(payload["from"], json!("gemma4:e2b"));
        assert_eq!(payload["system"], json!("New behavior"));
        assert_eq!(payload["template"], json!("template"));
        assert_eq!(payload["parameters"]["temperature"], json!(0.7));
        assert!(payload.get("license").is_none());
    }

    #[test]
    fn create_request_is_explicitly_non_streaming() {
        let payload = non_streaming(&json!({ "model": "gemma4:e2b" })).unwrap();

        assert_eq!(payload["stream"], json!(false));
    }

    #[test]
    fn non_object_create_payload_is_rejected() {
        assert!(non_streaming(&json!(["invalid"])).is_err());
    }
}
