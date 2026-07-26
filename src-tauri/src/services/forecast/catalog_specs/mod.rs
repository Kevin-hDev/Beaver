use super::{ForecastModelSpec, ForecastProviderSpec};

mod amazon;
mod datadog;
mod experimental;
mod google;
mod ibm;
mod nixtla;
mod providers;
mod salesforce;

pub(super) const TABPFN_TS_ALIAS: ForecastModelSpec = experimental::TABPFN_TS;

pub const FORECAST_PROVIDERS: &[ForecastProviderSpec] = providers::FORECAST_PROVIDERS;

pub const FORECAST_MODELS: &[ForecastModelSpec] = &[
    amazon::CHRONOS_BOLT_TINY,
    amazon::CHRONOS_BOLT_MINI,
    amazon::CHRONOS_BOLT_SMALL,
    amazon::CHRONOS_BOLT_BASE,
    amazon::CHRONOS_2,
    google::TIMESFM_2_5,
    nixtla::TIMEGPT_2_MINI,
    nixtla::TIMEGPT_2_STANDARD,
    nixtla::TIMEGPT_2_PRO,
    nixtla::TIMEGPT_2_1,
    datadog::TOTO_2_0_4M,
    datadog::TOTO_2_0_22M,
    datadog::TOTO_2_0_313M,
    datadog::TOTO_2_0_1B,
    datadog::TOTO_2_0_2_5B,
    salesforce::MOIRAI_2_0_R_SMALL,
    ibm::FLOWSTATE_R1,
    ibm::FLOWSTATE_R1_1,
    experimental::TABPFN_TS_3,
    experimental::TIREX,
    experimental::KAIROS_10M,
    experimental::KAIROS_23M,
    experimental::KAIROS_50M,
    experimental::SUNDIAL_128M,
];
