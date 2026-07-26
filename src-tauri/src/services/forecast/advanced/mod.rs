mod anomalies;
mod decomposition;
mod drift;
pub mod ensemble;
mod ensemble_combine;
mod sanitize;
mod stats;
mod types;
mod variable_samples;
mod variable_scoring;
mod variables;

pub use types::*;

use super::types::ForecastResult;

pub fn analyze(result: &ForecastResult) -> AdvancedAnalytics {
    let decomposition = decomposition::analyze(result);
    let mut analytics = AdvancedAnalytics {
        schema_version: 1,
        generated_at: chrono::Utc::now().to_rfc3339(),
        anomalies: anomalies::detect(&decomposition),
        variable_importance: variables::analyze(result),
        drift: drift::analyze(result),
        decomposition,
    };
    sanitize::apply(&mut analytics);
    analytics
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
