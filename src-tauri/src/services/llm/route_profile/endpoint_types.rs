#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) enum EndpointPolicy {
    Static {
        base_url: &'static str,
        models_endpoint: &'static str,
    },
    ConnectionConfigured,
    ProviderConnection {
        resolver: ConnectionEndpointResolver,
    },
    OllamaLocal,
}

impl EndpointPolicy {
    pub(in crate::services::llm) const fn static_parts(
        self,
    ) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Static {
                base_url,
                models_endpoint,
            } => Some((base_url, models_endpoint)),
            Self::ConnectionConfigured | Self::ProviderConnection { .. } | Self::OllamaLocal => {
                None
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) enum ConnectionEndpointResolver {
    QwenModelStudio,
}
