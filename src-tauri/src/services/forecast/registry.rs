use serde::Serialize;

#[path = "registry_caps.rs"]
mod registry_caps;
#[path = "registry_params.rs"]
mod registry_params;
#[path = "registry_specs.rs"]
mod registry_specs;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastEngineKind {
    LocalChronosBolt,
    LocalChronos2,
    LocalTimesFm,
    LocalToto,
    LocalMoirai,
    LocalFlowState,
    LocalTabPfnTs,
    LocalTiRex,
    LocalKairos,
    LocalSundial,
    CloudApi,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ForecastCapabilities {
    pub past_covariates: bool,
    pub future_covariates: bool,
    pub multi_series: bool,
    pub multivariate: bool,
    pub probabilistic: bool,
    pub backtesting_ready: bool,
    pub anomalies_ready: bool,
    pub fine_tuning_ready: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ForecastRuntimeSpec {
    pub model_id: &'static str,
    pub family_id: &'static str,
    pub engine_kind: ForecastEngineKind,
    pub capabilities: ForecastCapabilities,
    pub config_params: &'static [&'static str],
}

pub fn find_runtime(model_id: &str) -> Option<&'static ForecastRuntimeSpec> {
    FORECAST_RUNTIMES
        .iter()
        .find(|runtime| runtime.model_id == model_id)
}

pub fn is_cloud(runtime: &ForecastRuntimeSpec) -> bool {
    matches!(runtime.engine_kind, ForecastEngineKind::CloudApi)
}

pub fn has_predict_adapter(runtime: &ForecastRuntimeSpec) -> bool {
    matches!(
        runtime.engine_kind,
        ForecastEngineKind::LocalChronosBolt
            | ForecastEngineKind::LocalChronos2
            | ForecastEngineKind::LocalTimesFm
            | ForecastEngineKind::LocalToto
            | ForecastEngineKind::LocalMoirai
            | ForecastEngineKind::LocalFlowState
            | ForecastEngineKind::LocalTabPfnTs
            | ForecastEngineKind::LocalTiRex
            | ForecastEngineKind::LocalKairos
            | ForecastEngineKind::LocalSundial
            | ForecastEngineKind::CloudApi
    )
}

pub const FORECAST_RUNTIMES: &[ForecastRuntimeSpec] = registry_specs::FORECAST_RUNTIMES;
