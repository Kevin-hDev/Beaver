//! Détection des capacités (tools, thinking, vision) d'un modèle par provider.
//!
//! Patterns hardcodés pour les providers dont l'API n'expose pas les capabilities.
//! Ollama : détection dynamique via `/api/show` (ne pas utiliser ici).
//! OpenRouter : capacités lues depuis `/models`, pas depuis des patterns de nom.

fn strip_org_prefix(model_id: &str) -> &str {
    model_id
        .rsplit_once('/')
        .map(|(_, name)| name)
        .unwrap_or(model_id)
}

pub fn supports_tools(provider_id: &str, model_id: &str) -> bool {
    let model = strip_org_prefix(model_id).to_lowercase();
    match provider_id {
        "groq" => {
            model.starts_with("llama-3.3")
                || model.starts_with("llama-3.1")
                || model.starts_with("llama-4")
                || model.starts_with("mixtral")
                || model.starts_with("deepseek")
                || model.starts_with("gemma")
                || model.starts_with("qwen")
                || model.starts_with("compound")
                || model.starts_with("mistral")
        }
        "google" => {
            let has_gemini = model.contains("gemini");
            let is_pro = model.contains("pro");
            let is_flash_full = model.contains("flash") && !model.contains("flash-lite");
            is_gemma4_vision_model(&model) || (has_gemini && (is_pro || is_flash_full))
        }
        "mistral" => {
            model.starts_with("mistral-large")
                || model.starts_with("mistral-medium")
                || model.starts_with("mistral-small")
                || model.starts_with("codestral")
                || model.starts_with("devstral")
                || model.starts_with("magistral")
                || model.starts_with("open-mistral-nemo")
                || model.starts_with("ministral")
                || model.starts_with("pixtral")
        }
        "cerebras" => {
            model.starts_with("llama-3.1")
                || model.starts_with("llama-3.3")
                || model.starts_with("llama-4")
        }
        // OpenRouter : permissif, l'UI filtrera via flag supports_tools de l'API /models
        "openrouter" => true,
        "openai" => {
            model.starts_with("gpt-4")
                || model.starts_with("gpt-5")
                || model.starts_with("o3")
                || model.starts_with("o4-mini")
        }
        "deepseek" => model.starts_with("deepseek-chat") || model.starts_with("deepseek-v"),
        "xai" => super::providers::xai::supports_tools(&model),
        "moonshot" => super::providers::moonshot::supports_tools(&model),
        "zai" => super::providers::zai::supports_tools(&model),
        _ => false,
    }
}

/// Détection thinking/reasoning par patterns.
pub fn supports_thinking(provider_id: &str, model_id: &str) -> bool {
    let model = strip_org_prefix(model_id).to_lowercase();
    match provider_id {
        "deepseek" => super::providers::deepseek::supports_thinking(&model),
        "groq" => super::providers::groq::supports_thinking(&model),
        "openai" => {
            model.starts_with("o3") || model.starts_with("o4") || model.starts_with("gpt-5")
        }
        "google" => supports_google_thinking(&model),
        "openrouter" => false,
        "mistral" => super::providers::mistral::supports_thinking(&model),
        "xai" => super::providers::xai::supports_thinking(&model),
        "moonshot" => super::providers::moonshot::supports_thinking(&model),
        "zai" => super::providers::zai::supports_thinking(&model),
        _ => false,
    }
}

/// Détection vision par patterns.
pub fn supports_vision(provider_id: &str, model_id: &str) -> bool {
    let model = strip_org_prefix(model_id).to_lowercase();
    match provider_id {
        "groq" => model.starts_with("llama-4") || model.contains("vision"),
        "google" => model.contains("gemini") || is_gemma4_vision_model(&model),
        "mistral" => {
            model.starts_with("mistral-large")
                || model.starts_with("mistral-medium")
                || model.starts_with("mistral-small")
                || model.starts_with("ministral")
                || model.starts_with("pixtral")
        }
        "cerebras" => false,
        "openrouter" => is_gemma4_vision_model(&model),
        "openai" => {
            model.starts_with("gpt-4o")
                || model.starts_with("gpt-4-turbo")
                || model.starts_with("gpt-5")
                || model.starts_with("o4")
                || model.starts_with("o3")
        }
        "deepseek" => false,
        "xai" => super::providers::xai::supports_vision(&model),
        "moonshot" => super::providers::moonshot::supports_vision(&model),
        "zai" => super::providers::zai::supports_vision(&model),
        _ => false,
    }
}

fn is_gemma4_vision_model(model: &str) -> bool {
    model.starts_with("gemma-4")
        || model.starts_with("google/gemma-4")
        || model.contains("/gemma-4")
}

#[cfg(test)]
#[path = "tool_capable_tests.rs"]
mod tests;

fn supports_google_thinking(model: &str) -> bool {
    model.starts_with("gemini-2.5") || model.starts_with("gemini-3") || model.contains("thinking")
}
