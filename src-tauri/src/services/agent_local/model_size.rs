use super::system_prompt_types::PromptTier;

const MAX_COMPACT_B: f64 = 25.0;

pub fn detect_tier(model: &str) -> PromptTier {
    match extract_param_billions(model) {
        Some(b) if b <= MAX_COMPACT_B => PromptTier::Compact,
        Some(_) => PromptTier::Detailed,
        None => infer_from_keywords(model),
    }
}

/// Ollama metadata is authoritative; the model name is only a fallback for
/// older or incomplete `/api/show` responses.
pub fn detect_ollama_tier(parameter_size: &str, model: &str) -> PromptTier {
    parse_parameter_size(parameter_size).unwrap_or_else(|| detect_tier(model))
}

fn parse_parameter_size(parameter_size: &str) -> Option<PromptTier> {
    let normalized = parameter_size.trim().to_ascii_lowercase();
    let billions = if let Some(value) = normalized.strip_suffix('b') {
        value.trim().parse::<f64>().ok()?
    } else {
        let value = normalized.strip_suffix('m')?;
        value.trim().parse::<f64>().ok()? / 1_000.0
    };
    if !billions.is_finite() || billions < 0.0 {
        return None;
    }
    Some(if billions <= MAX_COMPACT_B {
        PromptTier::Compact
    } else {
        PromptTier::Detailed
    })
}

fn extract_param_billions(model: &str) -> Option<f64> {
    let lower = model.to_lowercase();
    for part in lower.split(|c: char| !c.is_alphanumeric() && c != '.') {
        if let Some(num_str) = part.strip_suffix('b') {
            if let Ok(size) = num_str.parse::<f64>() {
                if size.is_finite() && size >= 0.0 {
                    return Some(size);
                }
            }
        }
    }
    None
}

fn infer_from_keywords(model: &str) -> PromptTier {
    let lower = model.to_lowercase();
    let compact_keywords = [
        "small", "mini", "tiny", "nano", "micro", "e2b", "e4b", "lite",
    ];
    for kw in &compact_keywords {
        if lower.contains(kw) {
            return PromptTier::Compact;
        }
    }
    PromptTier::Detailed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_sizes() {
        assert_eq!(detect_tier("gemma-4-e4b"), PromptTier::Compact);
        assert_eq!(detect_tier("qwen3-7b"), PromptTier::Compact);
        assert_eq!(detect_tier("llama-3.3-8b"), PromptTier::Compact);
        assert_eq!(detect_tier("model-24b"), PromptTier::Compact);
        assert_eq!(detect_tier("model-25b"), PromptTier::Compact);
        assert_eq!(detect_tier("model-24.5b"), PromptTier::Compact);
        assert_eq!(detect_tier("model-25.5b"), PromptTier::Detailed);
        assert_eq!(detect_tier("qwen3-32b"), PromptTier::Detailed);
        assert_eq!(detect_tier("llama-3.3-70b"), PromptTier::Detailed);
        assert_eq!(detect_tier("mistral-small-3"), PromptTier::Compact);
        assert_eq!(detect_tier("mistral-large-3"), PromptTier::Detailed);
        assert_eq!(detect_tier("deepseek-chat"), PromptTier::Detailed);
        assert_eq!(detect_tier("gpt-5"), PromptTier::Detailed);
    }

    #[test]
    fn ollama_parameter_size_is_authoritative_over_the_model_name() {
        assert_eq!(
            detect_ollama_tier("7B", "misleading:70b"),
            PromptTier::Compact
        );
        assert_eq!(
            detect_ollama_tier("26B", "misleading:2b"),
            PromptTier::Detailed
        );
        assert_eq!(detect_ollama_tier("", "fallback:25b"), PromptTier::Compact);
    }
}
