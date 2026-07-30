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

#[derive(Debug, Clone, Default)]
pub struct ModelConfig {
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    pub max_output_tokens: Option<u32>,
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

fn map_provider_prefix(provider_id: &str) -> &str {
    match provider_id {
        "google" => "gemini",
        _ => provider_id,
    }
}

pub(crate) fn find_entry<'a>(
    registry: &'a HashMap<String, ModelEntry>,
    provider_id: &str,
    model_id: &str,
) -> Option<&'a ModelEntry> {
    let prefix = map_provider_prefix(provider_id);
    let key_prefixed = format!("{prefix}/{model_id}");
    registry
        .get(&key_prefixed)
        .or_else(|| matching_bare_entry(registry, model_id, prefix))
        .or_else(|| {
            let stripped = model_id
                .rsplit_once('/')
                .map(|(_, n)| n)
                .unwrap_or(model_id);
            let key2 = format!("{prefix}/{stripped}");
            registry
                .get(&key2)
                .or_else(|| matching_bare_entry(registry, stripped, prefix))
        })
}

fn matching_bare_entry<'a>(
    registry: &'a HashMap<String, ModelEntry>,
    model_id: &str,
    provider_prefix: &str,
) -> Option<&'a ModelEntry> {
    registry
        .get(model_id)
        .filter(|entry| entry.litellm_provider.as_deref() == Some(provider_prefix))
}

fn model_config(entry: &ModelEntry) -> Option<ModelConfig> {
    if !is_chat_mode(entry.mode.as_deref()) {
        return None;
    }
    Some(ModelConfig {
        supports_tools: entry.supports_function_calling,
        supports_vision: entry.supports_vision,
        supports_thinking: entry.supports_reasoning,
        max_output_tokens: positive_u32(entry.max_output_tokens.or(entry.max_tokens)),
    })
}

fn positive_u32(value: Option<u64>) -> Option<u32> {
    value
        .and_then(|tokens| u32::try_from(tokens).ok())
        .filter(|tokens| *tokens > 0)
}

pub async fn lookup(provider_id: &str, model_id: &str) -> Option<ModelConfig> {
    let registry = get_lock().read().await;
    find_entry(&registry, provider_id, model_id).and_then(model_config)
}

fn is_chat_mode(mode: Option<&str>) -> bool {
    matches!(mode, Some("chat") | Some("completion") | None)
}

pub async fn is_chat_model(provider_id: &str, model_id: &str) -> bool {
    let registry = get_lock().read().await;
    let entry = find_entry(&registry, provider_id, model_id);
    match entry {
        Some(e) => is_chat_mode(e.mode.as_deref()),
        None => !is_non_chat_name(model_id),
    }
}

pub(crate) fn is_trusted_host(host: &str) -> bool {
    host == "raw.githubusercontent.com"
}

pub(crate) fn is_body_size_ok(size: usize) -> bool {
    size <= MAX_BODY_BYTES
}

fn is_non_chat_name(model_id: &str) -> bool {
    let id = model_id.to_lowercase();
    let non_chat = [
        "whisper",
        "dall-e",
        "tts",
        "embedding",
        "embed",
        "moderation",
        "rerank",
        "lyria",
        "imagen",
        "veo",
        "music",
        "sora",
        "gpt-image",
        "stable-diffusion",
    ];
    non_chat.iter().any(|kw| id.contains(kw))
}

#[cfg(test)]
#[path = "model_registry_tests.rs"]
mod tests;
