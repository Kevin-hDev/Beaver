use super::model_registry::get_lock;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RegistryModelInfo {
    pub key: String,
    pub provider: String,
    pub mode: String,
    pub max_input_tokens: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub input_cost_per_mtok: Option<f64>,
    pub output_cost_per_mtok: Option<f64>,
    pub supports_vision: bool,
    pub supports_function_calling: bool,
    pub supports_reasoning: bool,
    pub supports_prompt_caching: bool,
    pub supports_audio_input: bool,
    pub supports_audio_output: bool,
    pub supports_web_search: bool,
    pub supports_response_schema: bool,
    pub supports_system_messages: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FamilyGroup {
    pub name: String,
    pub count: usize,
}

fn is_spec_key(key: &str) -> bool {
    matches!(key, "sample_spec")
}

pub async fn search(query: &str, limit: usize) -> Vec<RegistryModelInfo> {
    let reg = get_lock().read().await;
    let q = query.to_lowercase();
    let mut results = Vec::new();

    for (key, entry) in reg.iter() {
        if is_spec_key(key) || !key.to_lowercase().contains(&q) {
            continue;
        }
        results.push(to_info(key, entry));
        if results.len() >= limit {
            break;
        }
    }
    results.sort_by(|a, b| a.key.cmp(&b.key));
    results
}

pub async fn get_model(key: &str) -> Option<RegistryModelInfo> {
    let reg = get_lock().read().await;
    let entry = reg.get(key)?;
    Some(to_info(key, entry))
}

pub async fn list_families() -> Vec<FamilyGroup> {
    let reg = get_lock().read().await;
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for (key, entry) in reg.iter() {
        if is_spec_key(key) {
            continue;
        }
        let mode = entry.mode.as_deref().unwrap_or("chat");
        if !matches!(mode, "chat" | "completion") {
            continue;
        }
        let family = extract_family(key);
        *counts.entry(family).or_default() += 1;
    }

    let mut families: Vec<FamilyGroup> = counts
        .into_iter()
        .map(|(name, count)| FamilyGroup { name, count })
        .collect();
    families.sort_by_key(|b| std::cmp::Reverse(b.count));
    families
}

pub async fn list_family_models(family: &str) -> Vec<RegistryModelInfo> {
    let reg = get_lock().read().await;
    let mut results = Vec::new();

    for (key, entry) in reg.iter() {
        if is_spec_key(key) {
            continue;
        }
        if extract_family(key).as_str().ne(family) {
            continue;
        }
        results.push(to_info(key, entry));
    }
    results.sort_by(|a, b| a.key.cmp(&b.key));
    results
}

fn to_info(key: &str, e: &super::model_registry::ModelEntry) -> RegistryModelInfo {
    RegistryModelInfo {
        key: key.to_string(),
        provider: e.litellm_provider.clone().unwrap_or_default(),
        mode: e.mode.clone().unwrap_or_else(|| "chat".into()),
        max_input_tokens: e.max_input_tokens,
        max_output_tokens: e.max_output_tokens.or(e.max_tokens),
        input_cost_per_mtok: e.input_cost_per_token.map(|c| c * 1_000_000.0),
        output_cost_per_mtok: e.output_cost_per_token.map(|c| c * 1_000_000.0),
        supports_vision: e.supports_vision,
        supports_function_calling: e.supports_function_calling,
        supports_reasoning: e.supports_reasoning,
        supports_prompt_caching: e.supports_prompt_caching,
        supports_audio_input: e.supports_audio_input,
        supports_audio_output: e.supports_audio_output,
        supports_web_search: e.supports_web_search,
        supports_response_schema: e.supports_response_schema,
        supports_system_messages: e.supports_system_messages,
    }
}

fn extract_family(model_key: &str) -> String {
    let name = model_key
        .rsplit('/')
        .next()
        .unwrap_or(model_key)
        .to_lowercase();

    let families: &[(&[&str], &str)] = &[
        (&["gpt-4", "gpt-3", "chatgpt", "gpt-image"], "GPT"),
        (&["o1-", "o3-", "o4-"], "GPT"),
        (&["claude"], "Claude"),
        (&["gemini"], "Gemini"),
        (&["gemma"], "Gemma"),
        (&["llama"], "Llama"),
        (&["mistral"], "Mistral"),
        (&["mixtral"], "Mixtral"),
        (&["codestral"], "Codestral"),
        (&["pixtral"], "Pixtral"),
        (&["qwen"], "Qwen"),
        (&["deepseek"], "DeepSeek"),
        (&["phi-"], "Phi"),
        (&["command"], "Command"),
        (&["dall-e"], "DALL-E"),
        (&["whisper"], "Whisper"),
        (&["tts"], "TTS"),
        (&["nova"], "Nova"),
        (&["jamba"], "Jamba"),
        (&["grok"], "Grok"),
        (&["kimi", "moonshot"], "Kimi"),
        (&["glm"], "GLM"),
        (&["flux"], "Flux"),
    ];

    for (prefixes, family) in families {
        if prefixes.iter().any(|p| name.starts_with(p)) {
            return family.to_string();
        }
    }

    let first = name
        .split(|c: char| c.is_ascii_punctuation())
        .next()
        .unwrap_or(&name);
    let mut chars = first.chars();
    match chars.next() {
        Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
        None => "Other".to_string(),
    }
}
