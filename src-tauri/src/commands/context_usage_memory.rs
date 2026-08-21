use crate::services::agent_local::{
    memory_context_usage::MemoryContextUsage, system_prompt_types::PromptTier,
};
use std::path::Path;

pub struct ResolvedContextUsage {
    pub memory: MemoryContextUsage,
    pub prompt_tier: PromptTier,
}

pub async fn usage(
    provider: Option<&str>,
    model: &str,
    working_dir: &Path,
) -> ResolvedContextUsage {
    let context = if provider == Some("ollama") {
        crate::services::compress::context_resolve::resolve_ollama(model).await
    } else {
        let canonical =
            crate::services::llm::route::canonical_provider_id(provider.unwrap_or_default());
        crate::services::compress::context_resolve::resolve_api(canonical, model).await
    };
    // A single context resolution must drive both the runtime limit and prompt tier.
    let prompt_tier = prompt_tier_for_context(context.prompt_tier, model);
    let memory = crate::services::agent_local::memory_context::estimate_usage(
        working_dir,
        context.configured,
    )
    .await;
    ResolvedContextUsage {
        memory,
        prompt_tier,
    }
}

fn prompt_tier_for_context(resolved: Option<PromptTier>, model: &str) -> PromptTier {
    resolved.unwrap_or_else(|| {
        crate::services::agent_local::system_prompt_defaults::tier_for_model(model)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ollama_metadata_tier_wins_over_a_misleading_model_name() {
        assert_eq!(
            prompt_tier_for_context(Some(PromptTier::Compact), "custom-model:70b"),
            PromptTier::Compact
        );
    }
}
