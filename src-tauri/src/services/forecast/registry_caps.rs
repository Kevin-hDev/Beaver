use super::ForecastCapabilities;

const BASE_CAPABILITIES: ForecastCapabilities = ForecastCapabilities {
    past_covariates: false,
    future_covariates: false,
    multi_series: false,
    multivariate: false,
    probabilistic: false,
    backtesting_ready: false,
    anomalies_ready: false,
    fine_tuning_ready: false,
};

pub const fn chronos_bolt_caps() -> ForecastCapabilities {
    ForecastCapabilities {
        probabilistic: true,
        backtesting_ready: true,
        ..BASE_CAPABILITIES
    }
}

pub const fn chronos_2_caps() -> ForecastCapabilities {
    ForecastCapabilities {
        past_covariates: true,
        future_covariates: true,
        multi_series: true,
        probabilistic: true,
        backtesting_ready: true,
        ..BASE_CAPABILITIES
    }
}

pub const fn timesfm_caps() -> ForecastCapabilities {
    ForecastCapabilities {
        past_covariates: true,
        future_covariates: true,
        multi_series: true,
        probabilistic: true,
        backtesting_ready: true,
        ..BASE_CAPABILITIES
    }
}

pub const fn multiseries_prob_caps() -> ForecastCapabilities {
    ForecastCapabilities {
        multi_series: true,
        probabilistic: true,
        backtesting_ready: true,
        ..BASE_CAPABILITIES
    }
}

pub const fn toto_caps() -> ForecastCapabilities {
    ForecastCapabilities {
        multi_series: true,
        multivariate: true,
        probabilistic: true,
        backtesting_ready: true,
        ..BASE_CAPABILITIES
    }
}

pub const fn timegpt_caps(multivariate: bool) -> ForecastCapabilities {
    ForecastCapabilities {
        past_covariates: true,
        future_covariates: true,
        multi_series: true,
        multivariate,
        probabilistic: true,
        backtesting_ready: true,
        ..BASE_CAPABILITIES
    }
}
