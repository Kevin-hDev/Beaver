use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::RwLock;

static REGISTRY: OnceLock<RwLock<HashMap<String, ModelEntry>>> = OnceLock::new();

const EMBEDDED_JSON: &str = include_str!("../../../resources/litellm-models.json");
const MAX_REGISTRY_ENTRIES: usize = 3_500;
pub(crate) const MAX_BODY_BYTES: usize = 20 * 1024 * 1024; // 20 Mo max

#[derive(Debug, Clone, Deserialize)]
pub struct ModelEntry {
    pub litellm_provider: Option<String>,
    pub max_input_tokens: Option<u64>,
    pub max_output_tokens: Option<u64>,
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
    pub mode: Option<String>,
}

pub(crate) fn parse_registry(json: &str) -> HashMap<String, ModelEntry> {
    let raw: HashMap<String, serde_json::Value> = match serde_json::from_str(json) {
        Ok(m) => m,
        Err(_) => return HashMap::new(),
    };
    let cap = raw.len().min(MAX_REGISTRY_ENTRIES);
    let mut result = HashMap::with_capacity(cap);
    for (key, val) in raw {
        if result.len() >= MAX_REGISTRY_ENTRIES {
            eprintln!("[registry] borne atteinte ({MAX_REGISTRY_ENTRIES}), entrées excédentaires ignorées");
            break;
        }
        if let Ok(entry) = serde_json::from_value::<ModelEntry>(val) {
            result.insert(key, entry);
        }
    }
    result
}

pub(crate) fn get_lock() -> &'static RwLock<HashMap<String, ModelEntry>> {
    REGISTRY.get_or_init(|| {
        let data = super::model_registry_refresh::read_cache()
            .and_then(|s| {
                let map = parse_registry(&s);
                if map.len() > 100 {
                    Some(map)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| parse_registry(EMBEDDED_JSON));
        RwLock::new(data)
    })
}

pub async fn init() {
    let _ = get_lock();
    tokio::spawn(async { super::model_registry_refresh::refresh().await });
}

pub(crate) fn is_trusted_host(host: &str) -> bool {
    host == "raw.githubusercontent.com"
}

pub(crate) fn is_body_size_ok(size: usize) -> bool {
    size <= MAX_BODY_BYTES
}

#[cfg(test)]
#[path = "model_registry_tests.rs"]
mod tests;
