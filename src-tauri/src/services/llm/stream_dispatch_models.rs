use super::route_profile::{self, ClientSelector};

pub(crate) struct ModelRouteDescriptor {
    pub canonical_provider: &'static str,
    pub transport_family: &'static str,
    pub generation_supported: bool,
}

pub(crate) fn model_route_descriptor(route_id: &str) -> Option<ModelRouteDescriptor> {
    let profile = route_profile::find(route_id)?;
    let transport_family = match profile.client {
        ClientSelector::OpenAiCompat => "openaiChatCompletions",
        ClientSelector::OpenAiResponses => "openaiResponses",
        ClientSelector::Codex => "codexResponses",
        ClientSelector::OllamaLocal => "ollama",
        ClientSelector::Anthropic => "anthropicMessages",
        ClientSelector::XaiOauth => "xaiOAuth",
    };
    Some(ModelRouteDescriptor {
        canonical_provider: profile.canonical_provider.as_str(),
        transport_family,
        generation_supported: profile.availability.silent
            && !matches!(profile.client, ClientSelector::XaiOauth),
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn model_generation_route_matrix_follows_profiles() {
        for profile in super::route_profile::all() {
            let descriptor = super::model_route_descriptor(profile.id.provider_id())
                .expect("every profile has a descriptor");
            assert_eq!(
                descriptor.generation_supported,
                profile.availability.silent
                    && profile.client != super::route_profile::ClientSelector::XaiOauth,
                "{}",
                profile.id.provider_id(),
            );
        }
    }
}
