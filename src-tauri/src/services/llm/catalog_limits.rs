// LiteLLM currently publishes fewer than 4,000 entries. Ten thousand leaves
// growth room while rejecting an unbounded external map before publication.
pub(crate) const MAX_LITELLM_CATALOG_ENTRIES: usize = 10_000;

const MAX_DEFAULT_DYNAMIC_MODELS: usize = 500;
const MAX_OPENROUTER_DYNAMIC_MODELS: usize = 1_000;

pub(crate) fn max_dynamic_models(provider_id: &str) -> usize {
    if provider_id == "openrouter" {
        MAX_OPENROUTER_DYNAMIC_MODELS
    } else {
        MAX_DEFAULT_DYNAMIC_MODELS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_openrouter_has_the_larger_dynamic_catalog_limit() {
        assert_eq!(max_dynamic_models("openrouter"), 1_000);
        assert_eq!(max_dynamic_models("moonshot"), 500);
        assert_eq!(max_dynamic_models("openai"), 500);
    }
}
