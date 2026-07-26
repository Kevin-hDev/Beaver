use super::{limits, types::ForecastRequest};

const ALLOWED_REASON_CODES: &[&str] = &[
    "top_backtest",
    "beats_baseline",
    "precision_requested",
    "speed_requested",
    "local_required",
    "cloud_allowed",
    "user_requested",
    "resource_fit",
];

pub(super) fn validate_selection_metadata(request: &ForecastRequest) -> Result<(), String> {
    if request.selection_reason_codes.len() > limits::MAX_SELECTION_REASON_CODES {
        return Err("Trop de raisons de sélection".into());
    }
    let mut unique = std::collections::BTreeSet::new();
    if request
        .selection_reason_codes
        .iter()
        .any(|reason| !ALLOWED_REASON_CODES.contains(&reason.as_str()) || !unique.insert(reason))
    {
        return Err("Raisons de sélection invalides".into());
    }
    if request
        .selection_id
        .as_ref()
        .is_some_and(|id| id.chars().count() > 64 || uuid::Uuid::parse_str(id).is_err())
    {
        return Err("Sélection Auto invalide".into());
    }
    Ok(())
}
