use super::data_quality::DataProfile;
use super::types::ForecastRequest;

pub fn ensure_fingerprint(profile: &mut DataProfile, data: &str) {
    if !profile.fingerprint.is_empty() {
        return;
    }
    let request = ForecastRequest {
        data: Some(data.to_string()),
        file_path: None,
        data_profile_id: Some(profile.id.clone()),
        target_column: profile.target_column.clone(),
        date_column: profile.date_column.clone(),
        series_column: profile.series_column.clone(),
        covariate_columns: profile.covariate_columns.clone(),
        horizon: profile.horizon,
        frequency: profile.frequency.clone(),
        model: None,
        confidence_level: super::types::default_confidence(),
        selection_id: None,
        selection_source: None,
        selection_reason_codes: Vec::new(),
    };
    profile.fingerprint = super::data_fingerprint::for_request(&request);
}
