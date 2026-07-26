use std::path::Path;

pub async fn usage(
    provider: Option<&str>,
    model: &str,
    working_dir: &Path,
) -> crate::services::agent_local::memory_context_usage::MemoryContextUsage {
    let context = if provider == Some("ollama") {
        crate::services::compress::context_resolve::resolve_ollama(model).await
    } else {
        let canonical =
            crate::services::llm::route::canonical_provider_id(provider.unwrap_or_default());
        crate::services::compress::context_resolve::resolve_api(canonical, model).await
    };
    crate::services::agent_local::memory_context::estimate_usage(working_dir, context.configured)
        .await
}
