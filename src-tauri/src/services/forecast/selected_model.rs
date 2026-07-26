use super::{
    selection_policy::{self, ForecastSelectionMode},
    types::ForecastRequest,
    validation,
};

pub(crate) fn apply_policy(
    request: &mut ForecastRequest,
    policy: selection_policy::ForecastSelectionPolicy,
) -> Result<String, String> {
    match policy.mode {
        ForecastSelectionMode::Manual => {
            if request.selection_id.is_some()
                || request.selection_source.is_some()
                || !request.selection_reason_codes.is_empty()
            {
                return Err("Métadonnées Auto interdites en mode Manuel".into());
            }
            let model = policy
                .manual_model_id
                .ok_or_else(|| "Aucun modèle Forecast sélectionné".to_string())?;
            if request
                .model
                .as_deref()
                .is_some_and(|requested| requested != model)
            {
                return Err("Le modèle demandé ne correspond pas à la sélection manuelle".into());
            }
            request.model = Some(model.clone());
            Ok(model)
        }
        ForecastSelectionMode::Auto => {
            if request.selection_id.is_none() {
                return Err("Sélection Auto requise".into());
            }
            match request.selection_source {
                Some(super::provenance_types::ForecastSelectionSource::Auto) => {}
                Some(super::provenance_types::ForecastSelectionSource::ExplicitUserOverride)
                    if request
                        .selection_reason_codes
                        .iter()
                        .any(|reason| reason == "user_requested") => {}
                _ => return Err("Source de sélection Auto invalide".into()),
            }
            let model = request
                .model
                .as_deref()
                .ok_or_else(|| "Aucun modèle compatible choisi pour Auto".to_string())?;
            validation::validate_runnable_model_id(model)?;
            let spec = super::catalog::find_model(model).ok_or("Modèle inconnu")?;
            if spec.is_cloud && !policy.allow_cloud_in_auto {
                return Err("Les modèles cloud ne sont pas autorisés en mode Auto".into());
            }
            Ok(model.to_string())
        }
    }
}

pub(crate) fn apply_frontend_policy(
    request: &mut ForecastRequest,
    policy: selection_policy::ForecastSelectionPolicy,
) -> Result<String, String> {
    if policy.mode == ForecastSelectionMode::Manual {
        return apply_policy(request, policy);
    }
    let model = request
        .model
        .as_deref()
        .ok_or("Aucun modèle compatible choisi pour Auto")?;
    validation::validate_runnable_model_id(model)?;
    let spec = super::catalog::find_model(model).ok_or("Modèle inconnu")?;
    if spec.is_cloud && !policy.allow_cloud_in_auto {
        return Err("Les modèles cloud ne sont pas autorisés en mode Auto".into());
    }
    Ok(model.to_string())
}

#[cfg(test)]
#[path = "selected_model_tests.rs"]
mod tests;
