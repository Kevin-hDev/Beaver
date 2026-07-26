use crate::services::forecast::data_quality::DataProfile;
use crate::services::forecast::selection_policy::{ForecastSelectionMode, ForecastSelectionPolicy};
use crate::services::forecast::selection_tickets::SelectionProof;
use crate::services::forecast::types::{ForecastRequest, ForecastResult};
use crate::services::forecast::{
    auto_selection, hardware_profile, model_listing, selection_tickets, storage,
};

pub async fn verify(
    request: &ForecastRequest,
    profile: &DataProfile,
    policy: ForecastSelectionPolicy,
    session_id: &str,
) -> Result<Option<SelectionProof>, String> {
    if policy.mode == ForecastSelectionMode::Manual {
        return Ok(None);
    }
    let selection_id = request.selection_id.as_deref().ok_or("Sélection Auto requise")?;
    let model_id = request.model.as_deref().ok_or("Modèle Auto requis")?;
    let mut proof = selection_tickets::consume(
        selection_id,
        session_id,
        &profile.id,
        &profile.fingerprint,
        model_id,
    )?;
    let listing = model_listing::list_models();
    let models = listing["models"]
        .as_array()
        .ok_or("Catalogue Forecast indisponible")?;
    let evidence = storage::comparable_backtests(profile).await?;
    let requested_model = (request.selection_source
        == Some(
            crate::services::forecast::provenance_types::ForecastSelectionSource::ExplicitUserOverride,
        ))
    .then_some(model_id);
    let current = auto_selection::select_with_requested_model(
        models,
        profile,
        policy.allow_cloud_in_auto,
        hardware_profile::detect(),
        &evidence,
        requested_model,
    );
    let candidate = current
        .candidates
        .iter()
        .find(|candidate| candidate.model_id == model_id)
        .ok_or("Les candidats ou les ressources Forecast ont changé")?;
    proof.resource_fit = candidate.resource_fit;
    Ok(Some(proof))
}

pub fn complete_provenance(
    forecast: &mut ForecastResult,
    request: &ForecastRequest,
    profile: &DataProfile,
    proof: Option<&SelectionProof>,
    duration_ms: u64,
) -> Result<(), String> {
    let source = request.selection_source.unwrap_or(
        crate::services::forecast::provenance_types::ForecastSelectionSource::Manual,
    );
    crate::services::forecast::provenance::complete(
        forecast,
        request,
        profile,
        source,
        proof,
        duration_ms,
    )
}
