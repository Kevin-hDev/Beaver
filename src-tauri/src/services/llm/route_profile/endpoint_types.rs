#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "configurable endpoint variants are compiled before candidate route activation"
)]
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
    RegionAllowlist {
        regions: &'static [(&'static str, &'static str)],
    },
    Workspace {
        host_suffix: &'static str,
    },
    ValidatedHttps,
    PinnedBackend {
        base_url: &'static str,
    },
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
            Self::ConnectionConfigured
            | Self::ProviderConnection { .. }
            | Self::OllamaLocal
            | Self::RegionAllowlist { .. }
            | Self::Workspace { .. }
            | Self::ValidatedHttps
            | Self::PinnedBackend { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::services::llm) enum ConnectionEndpointResolver {
    QwenModelStudio,
}
