use super::data_quality::DataProfile;
use super::selection_policy::ForecastSelectionPolicy;
use super::selection_tickets::SelectionProof;
use super::{auto_selection, hardware_profile, model_listing, selection_tickets, storage};

pub async fn verify_choice(
    profile: &DataProfile,
    policy: &ForecastSelectionPolicy,
    model_id: &str,
) -> Result<SelectionProof, String> {
    let listing = model_listing::list_models();
    let models = listing["models"]
        .as_array()
        .ok_or("Catalogue Forecast indisponible")?;
    let evidence = storage::comparable_backtests(profile).await?;
    let selection = auto_selection::select_with_requested_model(
        models,
        profile,
        policy.allow_cloud_in_auto,
        hardware_profile::detect(),
        &evidence,
        Some(model_id),
    );
    let candidate = selection
        .candidates
        .iter()
        .find(|candidate| candidate.model_id == model_id)
        .ok_or("Le modèle ne fait pas partie des candidats Auto")?;
    Ok(selection_tickets::proof_for_candidate(
        selection.basis,
        candidate,
    ))
}
