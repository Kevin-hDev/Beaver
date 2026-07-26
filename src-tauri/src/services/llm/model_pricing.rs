use super::model_registry::{get_lock, ModelEntry};

#[derive(Debug, Clone, Copy)]
pub struct ModelPricing {
    pub input_cost_per_token: Option<f64>,
    pub output_cost_per_token: Option<f64>,
    pub cache_read_input_token_cost: Option<f64>,
}

pub async fn lookup(provider_id: &str, model_id: &str) -> Option<ModelPricing> {
    let registry = get_lock().read().await;
    let entry = find_entry(&registry, provider_id, model_id)?;
    Some(ModelPricing {
        input_cost_per_token: entry.input_cost_per_token,
        output_cost_per_token: entry.output_cost_per_token,
        cache_read_input_token_cost: entry.cache_read_input_token_cost,
    })
}

fn find_entry<'a>(
    registry: &'a std::collections::HashMap<String, ModelEntry>,
    provider_id: &str,
    model_id: &str,
) -> Option<&'a ModelEntry> {
    let prefix = if provider_id == "google" {
        "gemini"
    } else {
        provider_id
    };
    let prefixed = format!("{prefix}/{model_id}");
    registry
        .get(&prefixed)
        .or_else(|| matching_bare_entry(registry, model_id, prefix))
        .or_else(|| {
            let stripped = model_id
                .rsplit_once('/')
                .map(|(_, name)| name)
                .unwrap_or(model_id);
            registry
                .get(&format!("{prefix}/{stripped}"))
                .or_else(|| matching_bare_entry(registry, stripped, prefix))
        })
}

fn matching_bare_entry<'a>(
    registry: &'a std::collections::HashMap<String, ModelEntry>,
    model_id: &str,
    provider_prefix: &str,
) -> Option<&'a ModelEntry> {
    registry
        .get(model_id)
        .filter(|entry| entry.litellm_provider.as_deref() == Some(provider_prefix))
}

#[cfg(test)]
mod tests {
    use super::find_entry;

    #[test]
    fn bare_price_must_belong_to_the_requested_provider() {
        let registry = super::super::model_registry::parse_registry(
            r#"{
                "shared-model": {"litellm_provider":"openai","mode":"chat"},
                "xai/shared-model": {"litellm_provider":"xai","mode":"chat"}
            }"#,
        );

        assert_eq!(
            find_entry(&registry, "xai", "shared-model")
                .and_then(|entry| entry.litellm_provider.as_deref()),
            Some("xai")
        );
        assert!(find_entry(&registry, "moonshot", "shared-model").is_none());
        assert!(find_entry(&registry, "openai", "shared-model").is_some());
    }

    #[test]
    fn google_uses_the_gemini_registry_identity() {
        let registry = super::super::model_registry::parse_registry(
            r#"{"flash":{"litellm_provider":"gemini","mode":"chat"}}"#,
        );

        assert!(find_entry(&registry, "google", "flash").is_some());
    }
}
