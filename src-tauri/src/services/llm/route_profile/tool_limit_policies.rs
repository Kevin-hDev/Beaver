use super::{ResolvedToolLimitPolicy, RouteProfile, ToolLimitPolicy, UpstreamToolFamily};

pub(super) fn resolve(profile: &RouteProfile, model: &str) -> ResolvedToolLimitPolicy {
    let upstream = if profile.policies.tool_limits == ToolLimitPolicy::OpenRouterUpstream {
        upstream_family(model)
    } else {
        UpstreamToolFamily::Other
    };
    ResolvedToolLimitPolicy {
        policy: profile.policies.tool_limits,
        upstream,
    }
}

fn upstream_family(model: &str) -> UpstreamToolFamily {
    let family = model
        .split_once('/')
        .map(|(family, _)| family)
        .unwrap_or_default()
        .to_ascii_lowercase();
    match family.as_str() {
        "google" => UpstreamToolFamily::Google,
        "x-ai" | "xai" => UpstreamToolFamily::Xai,
        "mistralai" | "mistral" => UpstreamToolFamily::Mistral,
        "deepseek" => UpstreamToolFamily::DeepSeek,
        _ => UpstreamToolFamily::Other,
    }
}
