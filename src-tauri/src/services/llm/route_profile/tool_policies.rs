use super::{ExtensionToolPolicy, ResolvedToolPolicy, RouteProfile, SchemaPolicy};

pub(super) fn resolve(profile: &RouteProfile, model: &str) -> ResolvedToolPolicy {
    let normalized = model.to_ascii_lowercase();
    let schema = match profile.policies.schema {
        SchemaPolicy::Upstream if normalized.starts_with("google/") => SchemaPolicy::Google,
        SchemaPolicy::Upstream
            if normalized.starts_with("moonshotai/") || normalized.starts_with("kimi/") =>
        {
            SchemaPolicy::Kimi
        }
        SchemaPolicy::Upstream if normalized.starts_with("x-ai/") => SchemaPolicy::Xai,
        SchemaPolicy::Upstream => SchemaPolicy::Generic,
        policy => policy,
    };
    let extensions =
        if profile.id.provider_id() == "openrouter" && normalized.starts_with("groq/compound") {
            ExtensionToolPolicy::NoTools
        } else if profile.id.provider_id() == "openrouter" && normalized.starts_with("groq/") {
            ExtensionToolPolicy::WithoutExtensions
        } else {
            ExtensionToolPolicy::All
        };
    ResolvedToolPolicy {
        schema,
        strict: matches!(
            profile.id.provider_id(),
            "openai" | "codex-oauth" | "moonshot" | "deepseek"
        ) || schema == SchemaPolicy::Kimi,
        extensions,
    }
}
