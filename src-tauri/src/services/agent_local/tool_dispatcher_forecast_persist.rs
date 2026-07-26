use crate::services::forecast::types::{ForecastRequest, ForecastResult};
use crate::services::forecast::{data_profiles, storage};
use tokio_util::sync::CancellationToken;

pub async fn save(
    forecast: &mut ForecastResult,
    request: &ForecastRequest,
    cancel: &CancellationToken,
) -> Result<String, String> {
    ensure_active(cancel)?;
    if let Some(profile) = &forecast.data_profile {
        data_profiles::save(profile, request)
            .await
            .map_err(|_| "Sauvegarde du profil de données échouée".to_string())?;
    }
    ensure_active(cancel)?;
    storage::save(forecast)
        .await
        .map_err(|_| "Sauvegarde de la prévision échouée".to_string())?;
    if cancel.is_cancelled() {
        storage::delete(&forecast.id)
            .await
            .map_err(|_| "Annulation de la prévision incomplète".to_string())?;
        return Err("Annulé".to_string());
    }
    if let Some(app) = super::app_handle_global::get() {
        crate::services::forecast::events::emit_created(app, forecast);
    }
    super::tool_dispatcher_forecast_output::created_payload(forecast)
}

fn ensure_active(cancel: &CancellationToken) -> Result<(), String> {
    if cancel.is_cancelled() {
        Err("Annulé".to_string())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancelled_work_is_rejected_before_persistence() {
        let cancel = CancellationToken::new();
        cancel.cancel();

        assert_eq!(ensure_active(&cancel), Err("Annulé".to_string()));
    }
}
