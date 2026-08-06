use serde_json::Value;

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
    fn create_request_is_explicitly_non_streaming() {
        let payload = non_streaming(&json!({ "model": "gemma4:e2b" })).unwrap();

        assert_eq!(payload["stream"], json!(false));
    }

    #[test]
    fn non_object_create_payload_is_rejected() {
        assert!(non_streaming(&json!(["invalid"])).is_err());
    }
}
