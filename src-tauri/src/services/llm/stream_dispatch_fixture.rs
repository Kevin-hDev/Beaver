use super::{
    ensure_catalog_model, resolve_checked, route_profile, ClientSelector, RequestPurpose,
    ResolvedTransport, RouteSelectionError,
};
use crate::services::llm::route_profile::CatalogPolicy;

pub(crate) async fn resolve_fixture_transport(
    route_id: &str,
    model: &str,
    target: &crate::services::reasoning_continuity::contract::ContinuationTarget,
    purpose: RequestPurpose,
) -> Result<ResolvedTransport, RouteSelectionError> {
    if purpose != RequestPurpose::ManualChat || !target.is_fixture_candidate() {
        return Err(RouteSelectionError::Unavailable);
    }
    let replay = target.replay().ok_or(RouteSelectionError::Unavailable)?;
    let profile = route_profile::find(route_id).ok_or(RouteSelectionError::UnknownRoute)?;
    if !route_profile::supports_bounded_fixture(route_id) {
        return Err(RouteSelectionError::Unavailable);
    }
    let fixture_catalog = matches!(
        profile.catalog,
        CatalogPolicy::PublicApi { .. } | CatalogPolicy::ConfigurableApi { .. }
    ) || matches!(
        profile.client,
        ClientSelector::Codex | ClientSelector::XaiOauth
    );
    if !fixture_catalog
        || replay.route_id != profile.id
        || replay.model_id != model
        || replay.validate().is_err()
        || crate::services::reasoning_continuity::registry::replay_policy(replay).is_none()
    {
        return Err(RouteSelectionError::InvalidModel);
    }
    ensure_catalog_model(profile, model).await?;
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
