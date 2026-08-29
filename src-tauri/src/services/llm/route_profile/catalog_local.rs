use super::policies::*;
use super::types::*;
use crate::services::reasoning_continuity::contract::RouteId;

pub(super) const LOCAL_PROFILES: &[RouteProfile] = &[RouteProfile {
    id: RouteId::Ollama,
    canonical_provider: CanonicalProviderId::Ollama,
    display_name: "Ollama",
    client: ClientSelector::OllamaLocal,
    wire: OLLAMA_WIRE,
    auth: AuthKind::Local,
    endpoint: EndpointPolicy::OllamaLocal,
    availability: AVAILABLE_ANY,
    catalog: CatalogPolicy::Hidden,
    strict_model_allowlist: false,
    output_limits: OutputLimitPolicy {
        automatic: false,
        fallback: None,
    },
    policies: OLLAMA_LOCAL,
}];
