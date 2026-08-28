use super::request_purpose::RequestPurpose;
use super::route_profile::{self, ClientSelector, RouteProfile};
use crate::services::llm_oauth::{XaiBackend, XaiCatalogModel};
use crate::services::provider_usage::{UsageApiFormat, UsageContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InvocationKind {
    Interactive,
    Silent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ClientKind {
    ChatCompletions,
    Responses,
    XaiOauth(XaiBackend),
    Codex,
    OllamaLocal,
}

#[derive(Debug)]
pub(super) struct ResolvedTransport {
    pub profile: &'static RouteProfile,
    pub client: ClientKind,
    pub usage_api_format: UsageApiFormat,
    pub xai_catalog_model: Option<XaiCatalogModel>,
}

impl ResolvedTransport {
    pub(super) fn usage_context<'a>(&self, model: &'a str) -> UsageContext<'a> {
        UsageContext {
            canonical_provider_id: self.profile.canonical_provider.as_str(),
            model,
            api_format: self.usage_api_format,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RouteSelectionError {
    UnknownRoute,
    Unavailable,
    InvalidModel,
}

impl RouteSelectionError {
    pub(super) const fn code(self) -> &'static str {
        match self {
            Self::UnknownRoute | Self::InvalidModel => "provider_configuration_invalid",
            Self::Unavailable => "provider_access_unavailable",
        }
    }
}

pub(super) async fn resolve_transport(
    route_id: &str,
    model: &str,
    invocation: InvocationKind,
    purpose: RequestPurpose,
) -> Result<ResolvedTransport, RouteSelectionError> {
    let profile = checked_profile(route_id, invocation, purpose)?;
    let xai_model = if profile.client == ClientSelector::XaiOauth {
        Some(
            crate::services::llm_oauth::xai_catalog_model(model)
                .await
                .map_err(|_| RouteSelectionError::InvalidModel)?,
        )
    } else {
        None
    };
    resolve_checked(profile, xai_model)
}

pub(crate) fn is_available(
    route_id: &str,
    invocation: InvocationKind,
    purpose: RequestPurpose,
) -> bool {
    checked_profile(route_id, invocation, purpose).is_ok()
}

fn checked_profile(
    route_id: &str,
    invocation: InvocationKind,
    purpose: RequestPurpose,
) -> Result<&'static RouteProfile, RouteSelectionError> {
    let profile = route_profile::find(route_id).ok_or(RouteSelectionError::UnknownRoute)?;
    let invocation_allowed = match invocation {
        InvocationKind::Interactive => profile.availability.interactive,
        InvocationKind::Silent => profile.availability.silent,
    };
    let purpose_allowed = match purpose {
        RequestPurpose::ManualChat => profile.availability.interactive,
        RequestPurpose::ExternalChannel => profile.availability.external_channel,
        RequestPurpose::Automation => profile.availability.automation,
        RequestPurpose::AccountMetadata => profile.availability.account_metadata,
        RequestPurpose::Unknown => false,
    };
    if invocation_allowed && purpose_allowed {
        Ok(profile)
    } else {
        Err(RouteSelectionError::Unavailable)
    }
}

fn resolve_checked(
    profile: &'static RouteProfile,
    xai_catalog_model: Option<XaiCatalogModel>,
) -> Result<ResolvedTransport, RouteSelectionError> {
    let (client, usage_api_format) = match profile.client {
        ClientSelector::OpenAiCompat => (ClientKind::ChatCompletions, profile.wire.usage),
        ClientSelector::OpenAiResponses => (ClientKind::Responses, profile.wire.usage),
        ClientSelector::Codex => (ClientKind::Codex, profile.wire.usage),
        ClientSelector::OllamaLocal => (ClientKind::OllamaLocal, profile.wire.usage),
        ClientSelector::XaiOauth => {
            let backend = xai_catalog_model
                .as_ref()
                .ok_or(RouteSelectionError::InvalidModel)?
                .backend;
            let usage = match backend {
                XaiBackend::ChatCompletions => UsageApiFormat::ChatCompletions,
                XaiBackend::Responses => UsageApiFormat::Responses,
            };
            (ClientKind::XaiOauth(backend), usage)
        }
        ClientSelector::Anthropic => return Err(RouteSelectionError::UnknownRoute),
    };
    Ok(ResolvedTransport {
        profile,
        client,
        usage_api_format,
        xai_catalog_model,
    })
}

#[cfg(test)]
pub(super) fn resolve_transport_for_test(
    route_id: &str,
    invocation: InvocationKind,
    purpose: RequestPurpose,
    xai_catalog_model: Option<XaiCatalogModel>,
) -> Result<ResolvedTransport, RouteSelectionError> {
    resolve_checked(
        checked_profile(route_id, invocation, purpose)?,
        xai_catalog_model,
    )
}
