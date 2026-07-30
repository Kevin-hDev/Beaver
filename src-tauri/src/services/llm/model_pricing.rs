use super::model_registry::get_lock;
use super::model_registry_lookup::find_provider_entry;

#[derive(Debug, Clone, Copy)]
pub struct ModelPricing {
    pub input_cost_per_token: Option<f64>,
    pub output_cost_per_token: Option<f64>,
    pub cache_read_input_token_cost: Option<f64>,
}

pub async fn lookup(provider_id: &str, model_id: &str) -> Option<ModelPricing> {
    let registry = get_lock().read().await;
    let entry = find_provider_entry(&registry, provider_id, model_id)?;
    Some(ModelPricing {
        input_cost_per_token: entry.input_cost_per_token,
        output_cost_per_token: entry.output_cost_per_token,
        cache_read_input_token_cost: entry.cache_read_input_token_cost,
    })
}

#[cfg(test)]
mod tests {
    use super::find_provider_entry;

    #[test]
    fn bare_price_must_belong_to_the_requested_provider() {
        let registry = super::super::model_registry::parse_registry(
            r#"{
                "shared-model": {"litellm_provider":"openai","mode":"chat"},
                "xai/shared-model": {"litellm_provider":"xai","mode":"chat"}
            }"#,
        );

        assert_eq!(
            find_provider_entry(&registry, "xai", "shared-model")
                .and_then(|entry| entry.litellm_provider.as_deref()),
            Some("xai")
        );
        assert!(find_provider_entry(&registry, "moonshot", "shared-model").is_none());
        assert!(find_provider_entry(&registry, "openai", "shared-model").is_some());
    }

    #[test]
    fn google_uses_the_gemini_registry_identity() {
        let registry = super::super::model_registry::parse_registry(
            r#"{"flash":{"litellm_provider":"gemini","mode":"chat"}}"#,
        );

        assert!(find_provider_entry(&registry, "google", "flash").is_some());
    }
}
