use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::RwLock;

static CATALOG: OnceLock<RwLock<HashMap<String, ModelEntry>>> = OnceLock::new();

const EMBEDDED_JSON: &str = include_str!("../../../resources/litellm-models.json");
pub(crate) const MAX_BODY_BYTES: usize = 20 * 1024 * 1024; // 20 Mo max

pub(crate) use super::litellm_catalog_parser::parse_catalog;
#[cfg(test)]
pub(crate) use super::litellm_catalog_parser::CatalogParseError;

#[derive(Debug, Clone, Deserialize)]
pub struct ModelEntry {
    pub litellm_provider: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_token_count")]
    pub max_input_tokens: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_optional_token_count")]
    pub max_output_tokens: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_optional_token_count")]
    pub max_tokens: Option<u64>,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(default)]
    pub supports_function_calling: bool,
    #[serde(default)]
    pub supports_reasoning: bool,
    #[serde(default)]
    pub supports_prompt_caching: bool,
    #[serde(default)]
    pub supports_audio_input: bool,
    #[serde(default)]
    pub supports_audio_output: bool,
    #[serde(default)]
    pub supports_web_search: bool,
    #[serde(default)]
    pub supports_response_schema: bool,
    #[serde(default)]
    pub supports_system_messages: bool,
    pub input_cost_per_token: Option<f64>,
    pub output_cost_per_token: Option<f64>,
    pub cache_read_input_token_cost: Option<f64>,
    pub cache_creation_input_token_cost: Option<f64>,
    pub mode: Option<String>,
}

fn deserialize_optional_token_count<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Number>::deserialize(deserializer)?;
    let Some(value) = value else {
        return Ok(None);
    };
    if let Some(integer) = value.as_u64() {
        return Ok(Some(integer));
    }
    let float = value
        .as_f64()
        .filter(|number| number.is_finite() && *number >= 0.0 && number.fract() == 0.0)
        .ok_or_else(|| serde::de::Error::custom("invalid token count"))?;
    // u64::MAX rounds up to 2^64 as f64; equality would saturate the cast.
    if float >= u64::MAX as f64 {
        return Err(serde::de::Error::custom("invalid token count"));
    }
    Ok(Some(float as u64))
}

pub(crate) fn get_lock() -> &'static RwLock<HashMap<String, ModelEntry>> {
    CATALOG.get_or_init(|| {
        let data = super::litellm_catalog_refresh::read_cache()
            .and_then(|s| {
                let map = parse_catalog(&s).ok()?;
                if map.len() > 100 {
                    Some(map)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                parse_catalog(EMBEDDED_JSON).expect("embedded LiteLLM catalog must stay valid")
            });
        RwLock::new(data)
    })
}

pub async fn init() {
    let _ = get_lock();
    super::litellm_catalog_refresh::refresh().await;
}

pub(crate) fn is_trusted_host(host: &str) -> bool {
    host == "raw.githubusercontent.com"
}

pub(crate) fn is_body_size_ok(size: usize) -> bool {
    size <= MAX_BODY_BYTES
}

#[cfg(test)]
#[path = "litellm_catalog_tests.rs"]
mod tests;
