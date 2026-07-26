#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PromptTier {
    Compact,
    Detailed,
}

const THRESHOLD_B: u64 = 25;

pub fn detect_tier(model: &str) -> PromptTier {
    match extract_param_billions(model) {
        Some(b) if b < THRESHOLD_B => PromptTier::Compact,
        Some(_) => PromptTier::Detailed,
        None => infer_from_keywords(model),
    }
}

fn extract_param_billions(model: &str) -> Option<u64> {
    let lower = model.to_lowercase();
    for part in lower.split(|c: char| !c.is_alphanumeric()) {
        if let Some(num_str) = part.strip_suffix('b') {
            if let Ok(n) = num_str.parse::<u64>() {
                return Some(n);
            }
            if let Ok(f) = num_str.parse::<f64>() {
                return Some(f as u64);
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
        assert_eq!(detect_tier("qwen3-32b"), PromptTier::Detailed);
        assert_eq!(detect_tier("llama-3.3-70b"), PromptTier::Detailed);
        assert_eq!(detect_tier("mistral-small-3"), PromptTier::Compact);
        assert_eq!(detect_tier("mistral-large-3"), PromptTier::Detailed);
        assert_eq!(detect_tier("deepseek-chat"), PromptTier::Detailed);
        assert_eq!(detect_tier("gpt-5"), PromptTier::Detailed);
    }
}
