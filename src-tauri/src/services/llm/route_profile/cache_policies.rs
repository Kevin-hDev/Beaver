use super::{CachePolicy, ResolvedCachePolicy, RouteProfile};

pub(super) fn resolve<'a>(profile: &RouteProfile, model: &'a str) -> ResolvedCachePolicy<'a> {
    let kind = match profile.policies.cache {
        CachePolicy::OpenRouter if model == "google/gemini-3.8-flash" => {
            CachePolicy::OpenRouterGemini
        }
        CachePolicy::OpenAi56 if !super::super::providers::openai::is_gpt_56(model) => {
            CachePolicy::None
        }
        policy => policy,
    };
    ResolvedCachePolicy {
        route_id: profile.id.provider_id(),
        model,
        kind,
        include_usage: profile.policies.include_usage,
    }
}
