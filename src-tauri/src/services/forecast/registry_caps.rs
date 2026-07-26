use super::ForecastCapabilities;

pub const fn chronos_bolt_caps() -> ForecastCapabilities {
    caps(false, false, false, false, true, true, false, false)
}

pub const fn chronos_2_caps() -> ForecastCapabilities {
    caps(true, true, true, false, true, true, false, false)
}

pub const fn timesfm_caps() -> ForecastCapabilities {
    caps(true, true, true, false, true, true, false, false)
}

pub const fn multiseries_prob_caps() -> ForecastCapabilities {
    caps(false, false, true, false, true, true, false, false)
}

pub const fn toto_caps() -> ForecastCapabilities {
    caps(false, false, true, true, true, true, false, false)
}

pub const fn timegpt_caps(multivariate: bool) -> ForecastCapabilities {
    caps(true, true, true, multivariate, true, true, false, false)
}

const fn caps(
    past_covariates: bool,
    future_covariates: bool,
    multi_series: bool,
    multivariate: bool,
    probabilistic: bool,
    backtesting_ready: bool,
    anomalies_ready: bool,
    fine_tuning_ready: bool,
) -> ForecastCapabilities {
    ForecastCapabilities {
        past_covariates,
        future_covariates,
        multi_series,
        multivariate,
        probabilistic,
        backtesting_ready,
        anomalies_ready,
        fine_tuning_ready,
    }
}
